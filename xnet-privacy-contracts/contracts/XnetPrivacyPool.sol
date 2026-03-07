// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// ─────────────────────────────────────────────────────────────────────────────
// XnetPrivacyPool — ZKP-based Private XNC Transfers
//
// Based on the Tornado Cash architecture, adapted for the XNET EVM.
// Tornado Cash is licensed under MIT: github.com/tornadocash/tornado-core
//
// Protocol flow:
//   1. deposit(commitment) — XNC enters the pool, commitment is recorded on-chain.
//   2. withdraw(proof, nullifier, recipient) — Funds are withdrawn via ZK proof.
//
// The following data is never exposed on-chain:
//   - The identity of the depositor.
//   - The identity of the withdrawer.
//   - The link between a specific deposit and its corresponding withdrawal.
// ─────────────────────────────────────────────────────────────────────────────

/// Groth16 verifier interface.
/// A dedicated verifier is deployed for each denomination.
interface IVerifier {
    function verifyProof(
        uint[2] calldata a,
        uint[2][2] calldata b,
        uint[2] calldata c,
        uint[2] calldata input  // [root, nullifierHash]
    ) external view returns (bool);
}

/// Poseidon hash interface (ZK-friendly hash function).
interface IHasher {
    function poseidon(bytes32[2] calldata inputs)
        external pure returns (bytes32);
}

contract XnetPrivacyPool {

    // ─────────────────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────────────────

    /// The Groth16 verifier contract.
    IVerifier public immutable verifier;

    /// The Poseidon hasher (ZK-friendly, compatible with Circom).
    IHasher public immutable hasher;

    /// The fixed deposit amount for this pool (sets the anonymity set).
    /// Typically: 1 XNC, 10 XNC, 100 XNC — each requires a separate pool.
    uint256 public immutable denomination;

    /// Merkle tree depth — 2^20 supports up to 1,048,576 deposits.
    uint32 public constant MERKLE_TREE_HEIGHT = 20;

    /// The operator address (receives the relayer fee).
    address public operator;

    // Merkle tree state
    uint256 public currentRootIndex;
    uint256 public nextIndex;

    /// Caches the most recent ROOT_HISTORY_SIZE roots.
    /// Enables users to withdraw using slightly older roots to avoid front-running.
    uint32 public constant ROOT_HISTORY_SIZE = 100;
    bytes32[ROOT_HISTORY_SIZE] public roots;

    /// Tracks whether a commitment has already been deposited.
    mapping(bytes32 => bool) public commitments;

    /// Tracks whether a nullifier has been spent (double-spend protection).
    mapping(bytes32 => bool) public nullifierHashes;

    /// Filled subtrees used for the incremental Merkle tree logic.
    bytes32[MERKLE_TREE_HEIGHT] public filledSubtrees;

    /// Zero values for empty leaves at each depth.
    bytes32[MERKLE_TREE_HEIGHT + 1] public zeros;

    // ─────────────────────────────────────────────────────────────────────────
    // Events
    // ─────────────────────────────────────────────────────────────────────────

    event Deposit(
        bytes32 indexed commitment,
        uint32 leafIndex,
        uint256 timestamp
    );

    event Withdrawal(
        address to,
        bytes32 nullifierHash,
        address indexed relayer,
        uint256 fee
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Constructor
    // ─────────────────────────────────────────────────────────────────────────

    constructor(
        IVerifier _verifier,
        IHasher _hasher,
        uint256 _denomination,
        address _operator
    ) {
        require(_denomination > 0, "Denomination must be > 0");
        verifier = _verifier;
        hasher = _hasher;
        denomination = _denomination;
        operator = _operator;

        // Precompute zero values recursively (Poseidon(0, 0))
        bytes32 currentZero = bytes32(0);
        zeros[0] = currentZero;
        filledSubtrees[0] = currentZero;

        for (uint32 i = 1; i < MERKLE_TREE_HEIGHT; i++) {
            currentZero = _hashPair(currentZero, currentZero);
            zeros[i] = currentZero;
            filledSubtrees[i] = currentZero;
        }

        // Setup the genesis root
        roots[0] = _hashPair(currentZero, currentZero);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Deposit
    // ─────────────────────────────────────────────────────────────────────────

    /// Shields XNC by inserting a commitment into the anonymity pool.
    ///
    /// @param commitment   Hash(secret, nullifier) — computed by the user off-chain.
    ///
    /// Client-side flow:
    /// 1. Generate random generic bytes: secret = random(), nullifier = random()
    /// 2. Compute commitment: commitment = poseidon([secret, nullifier])
    /// 3. Call deposit(commitment) providing the fixed denomination in msg.value.
    /// 4. Securely store the secret and nullifier (recovering funds is impossible without them).
    function deposit(bytes32 commitment) external payable {
        require(msg.value == denomination, "Wrong XNC amount");
        require(!commitments[commitment], "Commitment already exists");
        require(nextIndex < 2**MERKLE_TREE_HEIGHT, "Merkle tree is full");

        uint32 insertedIndex = _insert(commitment);
        commitments[commitment] = true;

        emit Deposit(commitment, insertedIndex, block.timestamp);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Withdraw
    // ─────────────────────────────────────────────────────────────────────────

    /// Withdraws shielded XNC using a valid zero-knowledge proof.
    ///
    /// @param proofA        Groth16 proof parameter A
    /// @param proofB        Groth16 proof parameter B
    /// @param proofC        Groth16 proof parameter C
    /// @param root          The Merkle root (must exist in recent history)
    /// @param nullifierHash Hash(nullifier, leafIndex) — strictly single-use
    /// @param recipient     The destination address receiving the XNC
    /// @param relayer       The relayer executing the transaction (for gasless UX)
    /// @param fee           The relayer fee (denominated in XNC)
    ///
    /// Anonymity protections:
    /// - The recipient is completely decoupled from the original depositor.
    /// - Executing via a relayer obscures the recipient's IP address.
    /// - The underlying secret cannot be reverse-engineered from the nullifierHash.
    function withdraw(
        uint[2] calldata proofA,
        uint[2][2] calldata proofB,
        uint[2] calldata proofC,
        bytes32 root,
        bytes32 nullifierHash,
        address payable recipient,
        address payable relayer,
        uint256 fee
    ) external {
        require(fee < denomination, "Fee exceeds denomination");
        require(!nullifierHashes[nullifierHash], "Already spent");
        require(isKnownRoot(root), "Unknown root");

        // Verify the ZK proof against the public inputs
        // Public inputs: [root, nullifierHash]
        require(
            verifier.verifyProof(
                proofA,
                proofB,
                proofC,
                [uint256(root), uint256(nullifierHash)]
            ),
            "Invalid ZK proof"
        );

        // Invalidate the nullifier (MUST execute before transfers to prevent reentrancy)
        nullifierHashes[nullifierHash] = true;

        // Disburse the XNC
        uint256 amount = denomination - fee;
        recipient.transfer(amount);

        if (fee > 0) {
            relayer.transfer(fee);
        }

        emit Withdrawal(recipient, nullifierHash, relayer, fee);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Merkle Tree (Incremental)
    // ─────────────────────────────────────────────────────────────────────────

    /// Inserts a new leaf into the incremental Merkle tree and updates the active root.
    function _insert(bytes32 leaf) internal returns (uint32 index) {
        uint32 _nextIndex = uint32(nextIndex);
        require(_nextIndex < 2**MERKLE_TREE_HEIGHT, "Tree is full");

        uint32 currentIndex = _nextIndex;
        bytes32 currentLevelHash = leaf;
        bytes32 left;
        bytes32 right;

        for (uint32 i = 0; i < MERKLE_TREE_HEIGHT; i++) {
            if (currentIndex % 2 == 0) {
                left = currentLevelHash;
                right = zeros[i];
                filledSubtrees[i] = currentLevelHash;
            } else {
                left = filledSubtrees[i];
                right = currentLevelHash;
            }
            currentLevelHash = _hashPair(left, right);
            currentIndex /= 2;
        }

        // Store the newly computed root in the ring buffer
        uint256 newRootIndex = (currentRootIndex + 1) % ROOT_HISTORY_SIZE;
        currentRootIndex = newRootIndex;
        roots[newRootIndex] = currentLevelHash;
        nextIndex = _nextIndex + 1;

        return _nextIndex;
    }

    function _hashPair(bytes32 left, bytes32 right)
        internal view returns (bytes32)
    {
        return hasher.poseidon([left, right]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // View functions
    // ─────────────────────────────────────────────────────────────────────────

    /// Validates whether a provided root exists within the recent cache history.
    function isKnownRoot(bytes32 root) public view returns (bool) {
        if (root == bytes32(0)) return false;
        uint256 i = currentRootIndex;
        do {
            if (root == roots[i]) return true;
            if (i == 0) i = ROOT_HISTORY_SIZE;
            i--;
        } while (i != currentRootIndex);
        return false;
    }

    /// Returns the active (most recent) Merkle root.
    function getLastRoot() external view returns (bytes32) {
        return roots[currentRootIndex];
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// XnetPrivacyPoolFactory — Factory for deploying multi-denomination pools
// ─────────────────────────────────────────────────────────────────────────────

contract XnetPrivacyPoolFactory {
    IVerifier public immutable verifier;
    IHasher public immutable hasher;

    // Mapping of denomination limits to their respective pool addresses
    mapping(uint256 => address) public pools;
    address[] public allPools;

    event PoolCreated(uint256 denomination, address pool);

    constructor(IVerifier _verifier, IHasher _hasher) {
        verifier = _verifier;
        hasher = _hasher;
    }

    /// Deploys a new anonymity pool for a specific token denomination.
    /// e.g., Dedicated pools for 1 XNC, 10 XNC, 100 XNC, 1000 XNC.
    function createPool(uint256 denomination) external returns (address) {
        require(pools[denomination] == address(0), "Pool exists");

        XnetPrivacyPool pool = new XnetPrivacyPool(
            verifier,
            hasher,
            denomination,
            msg.sender
        );

        pools[denomination] = address(pool);
        allPools.push(address(pool));

        emit PoolCreated(denomination, address(pool));
        return address(pool);
    }

    function getAllPools() external view returns (address[] memory) {
        return allPools;
    }
}
