// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// ─────────────────────────────────────────────────────────────────────────────
// XnetPrivacyPool — ZKP asosida maxfiy XNC transfer
//
// Tornado Cash arxitekturasi asosida, XNET EVM uchun moslashtirilgan.
// Tornado Cash MIT litsenziyada: github.com/tornadocash/tornado-core
//
// Qanday ishlaydi:
//   1. deposit(commitment) — XNC pool'ga tushadi, commitment on-chain
//   2. withdraw(proof, nullifier, recipient) — ZK proof bilan chiqariladi
//
// On-chain hech qachon ko'rinmaydi:
//   - Kim deposit qildi
//   - Kim withdraw qildi
//   - Qaysi deposit qaysi withdraw bilan bog'liq
// ─────────────────────────────────────────────────────────────────────────────

/// Groth16 verifier interfeysi
/// Har bir denomination uchun alohida verifier deploy qilinadi
interface IVerifier {
    function verifyProof(
        uint[2] calldata a,
        uint[2][2] calldata b,
        uint[2] calldata c,
        uint[2] calldata input  // [root, nullifierHash]
    ) external view returns (bool);
}

/// Poseidon hash interfeysi (ZK-friendly hash)
interface IHasher {
    function poseidon(bytes32[2] calldata inputs)
        external pure returns (bytes32);
}

contract XnetPrivacyPool {

    // ─────────────────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────────────────

    /// Groth16 verifier contract
    IVerifier public immutable verifier;

    /// Poseidon hasher (ZK-friendly, Circom bilan mos)
    IHasher public immutable hasher;

    /// Har bir deposit shu miqdorda bo'ladi (anonymity set uchun)
    /// Masalan: 1 XNC, 10 XNC, 100 XNC — alohida pool'lar
    uint256 public immutable denomination;

    /// Merkle tree chuqurligi — 2^20 = 1,048,576 deposit sig'adi
    uint32 public constant MERKLE_TREE_HEIGHT = 20;

    /// Operator manzili (relayer to'lovini oladi)
    address public operator;

    // Merkle tree
    uint256 public currentRootIndex;
    uint256 public nextIndex;

    /// Oxirgi ROOT_HISTORY_SIZE ta root saqlanadi
    /// Foydalanuvchi eski root bilan ham withdraw qila oladi
    uint32 public constant ROOT_HISTORY_SIZE = 100;
    bytes32[ROOT_HISTORY_SIZE] public roots;

    /// Commitment → deposit bo'ldimi
    mapping(bytes32 => bool) public commitments;

    /// Nullifier → sarflandimi (double-spend himoyasi)
    mapping(bytes32 => bool) public nullifierHashes;

    /// Merkle tree filledSubtrees (incremental tree uchun)
    bytes32[MERKLE_TREE_HEIGHT] public filledSubtrees;

    /// Zero values (bo'sh barglar uchun)
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

        // Zero value'larni hisoblash (Poseidon(0, 0) recursively)
        bytes32 currentZero = bytes32(0);
        zeros[0] = currentZero;
        filledSubtrees[0] = currentZero;

        for (uint32 i = 1; i < MERKLE_TREE_HEIGHT; i++) {
            currentZero = _hashPair(currentZero, currentZero);
            zeros[i] = currentZero;
            filledSubtrees[i] = currentZero;
        }

        // Dastlabki root
        roots[0] = _hashPair(currentZero, currentZero);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Deposit
    // ─────────────────────────────────────────────────────────────────────────

    /// XNC'ni shielded pool'ga yashirish.
    ///
    /// @param commitment   Hash(secret, nullifier) — foydalanuvchi hisoblaydi
    ///
    /// Foydalanuvchi qadamlari:
    /// 1. JavaScript'da: secret = random(), nullifier = random()
    /// 2. commitment = poseidon([secret, nullifier])
    /// 3. deposit(commitment) — bu function'ni chaqiradi
    /// 4. secret va nullifier'ni xavfsiz saqlaydi (hech kim bilmasligi kerak)
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

    /// ZK proof bilan XNC'ni chiqarish.
    ///
    /// @param proofA       Groth16 proof A
    /// @param proofB       Groth16 proof B
    /// @param proofC       Groth16 proof C
    /// @param root         Merkle root (oxirgi 100 ta rootdan biri)
    /// @param nullifierHash Hash(nullifier, leafIndex) — bir marta ishlatiladi
    /// @param recipient    XNC qabul qiluvchi (deposit qiluvchidan farq qiladi!)
    /// @param relayer      Relayer manzili (gasless tx uchun)
    /// @param fee          Relayer'ga to'lov (XNC)
    ///
    /// Anonymity qanday ta'minlanadi:
    /// - recipient deposit qiluvchi bilan bog'liq emas
    /// - relayer orqali yuborilsa — recipient'ning IP ham yashirin
    /// - nullifierHash'dan secret chiqarib bo'lmaydi
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

        // ZK proof tekshirish
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

        // Nullifier sarflandi (TRANSFER DAN OLDIN — reentrancy himoyasi)
        nullifierHashes[nullifierHash] = true;

        // XNC yuborish
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

    /// Yangi bargni Merkle tree'ga qo'shish va yangi rootni qaytarish.
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

        // Yangi rootni saqlash (ring buffer)
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

    /// Root oxirgi 100 ta rootdan birimi?
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

    /// Hozirgi Merkle root
    function getLastRoot() external view returns (bytes32) {
        return roots[currentRootIndex];
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// XnetPrivacyPoolFactory — ko'p denominationli pool'lar yaratish
// ─────────────────────────────────────────────────────────────────────────────

contract XnetPrivacyPoolFactory {
    IVerifier public immutable verifier;
    IHasher public immutable hasher;

    // denomination → pool address
    mapping(uint256 => address) public pools;
    address[] public allPools;

    event PoolCreated(uint256 denomination, address pool);

    constructor(IVerifier _verifier, IHasher _hasher) {
        verifier = _verifier;
        hasher = _hasher;
    }

    /// Yangi denomination uchun pool yaratish.
    /// Masalan: 1 XNC, 10 XNC, 100 XNC, 1000 XNC
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
