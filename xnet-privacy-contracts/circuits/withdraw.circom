// withdraw.circom — XNET Privacy Pool uchun Circom circuit
//
// Bu circuit foydalanuvchi deposit qilganligini isbotlaydi
// secret va nullifier'ni oshkor etmasdan.
//
// O'rnatish:
//   npm install -g circom snarkjs
//
// Compile:
//   circom withdraw.circom --r1cs --wasm --sym
//
// Trusted setup (Plonk — universal, ceremony shart emas):
//   snarkjs plonk setup withdraw.r1cs pot12_final.ptau circuit_plonk.zkey
//   snarkjs zkey export verificationkey circuit_plonk.zkey verification_key.json
//
// Proof yaratish:
//   snarkjs plonk prove circuit_plonk.zkey witness.wtns proof.json public.json
//
// Solidity verifier:
//   snarkjs zkey export solidityverifier circuit_plonk.zkey verifier.sol

pragma circom 2.0.0;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/bitify.circom";
include "node_modules/circomlib/circuits/merkleproof.circom";

// Merkle tree membership isboti
template MerkleTreeChecker(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component selectors[levels];
    component hashers[levels];

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== i == 0 ? leaf : hashers[i-1].out;
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== selectors[i].out[0];
        hashers[i].inputs[1] <== selectors[i].out[1];
    }

    root === hashers[levels-1].out;
}

// XNET Withdraw circuit
//
// YASHIRIN (private) kirish — hech kim bilmaydi:
//   - secret: random 32 bayt
//   - nullifier: random 32 bayt
//   - pathElements: Merkle yo'l elementlari
//   - pathIndices: Merkle yo'l indekslari
//
// OSHKOR (public) kirish — on-chain ko'rinadi:
//   - root: Merkle tree root
//   - nullifierHash: Hash(nullifier) — double-spend himoyasi
//   - recipient: XNC qabul qiluvchi
//   - fee: relayer to'lovi

template Withdraw(levels) {
    // === YASHIRIN KIRISH ===
    signal input secret;
    signal input nullifier;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // === OSHKOR KIRISH ===
    signal input root;
    signal input nullifierHash;
    signal input recipient;     // yashirish shart emas, lekin privacy uchun hash
    signal input fee;

    // === 1. Commitment hisoblash ===
    // commitment = Poseidon(secret, nullifier)
    component commitmentHasher = Poseidon(2);
    commitmentHasher.inputs[0] <== secret;
    commitmentHasher.inputs[1] <== nullifier;

    // === 2. NullifierHash tekshirish ===
    // nullifierHash = Poseidon(nullifier)
    component nullifierHasher = Poseidon(1);
    nullifierHasher.inputs[0] <== nullifier;
    nullifierHasher.out === nullifierHash;  // public input bilan mos bo'lishi shart

    // === 3. Merkle tree membership isboti ===
    // commitment Merkle tree'da bor ekanligini isbotlaymiz
    component tree = MerkleTreeChecker(levels);
    tree.leaf <== commitmentHasher.out;
    tree.root <== root;
    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }

    // === 4. Recipient va fee'ni bog'lash ===
    // Bu foydalanuvchi boshqa recipientga withdraw qila olmasligi uchun
    // (malleability himoyasi)
    signal recipientSquare;
    signal feeSquare;
    recipientSquare <== recipient * recipient;
    feeSquare <== fee * fee;
}

// 20 darajali Merkle tree (1,048,576 deposit)
component main {public [root, nullifierHash, recipient, fee]} =
    Withdraw(20);
