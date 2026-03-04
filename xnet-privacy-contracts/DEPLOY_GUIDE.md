# XNET Privacy Pool — To'liq Deploy Yo'riqnomasi

## Arxitektura

```
[Foydalanuvchi]
    │
    ├── JavaScript SDK (proof yaratadi)
    │       secret = random()
    │       nullifier = random()
    │       commitment = poseidon([secret, nullifier])
    │
    ├── deposit(commitment) ──→ [XnetPrivacyPool.sol]
    │       XNC pool'ga tushadi
    │       commitment Merkle tree'ga qo'shiladi
    │
    └── (istalgan vaqt, istalgan qurilma)
        withdraw(proof, nullifier, recipient) ──→ [XnetPrivacyPool.sol]
                ZK proof tekshiriladi
                XNC recipient'ga yuboriladi
                On-chain: kim = yo'q, qancha = yo'q
```

---

## 1-QADAM: Circuit Compile

```bash
# circom va snarkjs o'rnatish
npm install -g circom snarkjs

# Circuit compile
circom withdraw.circom --r1cs --wasm --sym -o ./build

# Powers of Tau (trusted setup — Plonk uchun universal, bir marta)
# Tayyor pot fayl ishlatamiz (Hermez tomonidan):
wget https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_12.ptau

# Plonk setup (ceremony shart emas!)
snarkjs plonk setup build/withdraw.r1cs powersOfTau28_hez_final_12.ptau circuit.zkey

# Verification key export
snarkjs zkey export verificationkey circuit.zkey vk.json

# Solidity verifier generatsiya
snarkjs zkey export solidityverifier circuit.zkey contracts/Verifier.sol
```

---

## 2-QADAM: Poseidon Hasher Deploy

```javascript
// Poseidon hasher — circomlibdan olinadi
// npm install circomlibjs

const { buildPoseidon } = require("circomlibjs");

// Yoki tayyor Solidity versiyasi:
// https://github.com/iden3/circomlibjs/blob/main/src/poseidon_opt.js
```

XNET testnet'ga deploy:
```bash
# Hardhat konfiguratsiyasi
# hardhat.config.js:
networks: {
  xnet_testnet: {
    url: "http://127.0.0.1:8545",  // local dev
    chainId: 2009,
    accounts: ["0x..."]  // test kalit
  }
}

# Deploy
npx hardhat run scripts/deploy.js --network xnet_testnet
```

---

## 3-QADAM: Deploy Script

```javascript
// scripts/deploy.js
const { ethers } = require("hardhat");

async function main() {
    const [deployer] = await ethers.getSigners();
    console.log("Deploying from:", deployer.address);

    // 1. Verifier deploy (snarkjs generatsiya qilgan)
    const Verifier = await ethers.getContractFactory("PlonkVerifier");
    const verifier = await Verifier.deploy();
    console.log("Verifier:", verifier.address);

    // 2. Poseidon Hasher deploy
    const Hasher = await ethers.getContractFactory("PoseidonHasher");
    const hasher = await Hasher.deploy();
    console.log("Hasher:", hasher.address);

    // 3. Privacy Pool deploylar — har bir denomination uchun
    const XnetPrivacyPool = await ethers.getContractFactory("XnetPrivacyPool");

    // 1 XNC pool
    const pool1 = await XnetPrivacyPool.deploy(
        verifier.address,
        hasher.address,
        ethers.utils.parseEther("1"),   // 1 XNC
        deployer.address
    );
    console.log("1 XNC Pool:", pool1.address);

    // 10 XNC pool
    const pool10 = await XnetPrivacyPool.deploy(
        verifier.address,
        hasher.address,
        ethers.utils.parseEther("10"),  // 10 XNC
        deployer.address
    );
    console.log("10 XNC Pool:", pool10.address);

    // 100 XNC pool
    const pool100 = await XnetPrivacyPool.deploy(
        verifier.address,
        hasher.address,
        ethers.utils.parseEther("100"), // 100 XNC
        deployer.address
    );
    console.log("100 XNC Pool:", pool100.address);
}

main().catch(console.error);
```

---

## 4-QADAM: JavaScript SDK (foydalanuvchi uchun)

```javascript
// xnet-privacy-sdk.js
const { buildPoseidon } = require("circomlibjs");
const { ethers } = require("ethers");
const snarkjs = require("snarkjs");

class XnetPrivacy {
    constructor(poolAddress, providerUrl) {
        this.provider = new ethers.providers.JsonRpcProvider(providerUrl);
        this.poolAddress = poolAddress;
    }

    // DEPOSIT QILISH
    async deposit(denomination) {
        const poseidon = await buildPoseidon();

        // 1. Random secret va nullifier yaratish
        const secret = ethers.utils.randomBytes(32);
        const nullifier = ethers.utils.randomBytes(32);

        // 2. Commitment hisoblash
        const commitment = poseidon.F.toString(
            poseidon([secret, nullifier])
        );

        // 3. NOTE saqlash (foydalanuvchi o'zi saqlaydi!)
        const note = {
            secret: ethers.utils.hexlify(secret),
            nullifier: ethers.utils.hexlify(nullifier),
            commitment: commitment,
            poolAddress: this.poolAddress,
            denomination: denomination,
            timestamp: Date.now()
        };

        console.log("NOTE (xavfsiz saqlang!):", JSON.stringify(note));
        console.log("NOTE string:", `xnet-${denomination}-${note.secret}-${note.nullifier}`);

        // 4. Deposit tranzaksiya
        const pool = new ethers.Contract(this.poolAddress, POOL_ABI, this.signer);
        const tx = await pool.deposit(commitment, {
            value: ethers.utils.parseEther(denomination.toString())
        });

        return { note, tx };
    }

    // WITHDRAW QILISH
    async withdraw(noteString, recipient) {
        // 1. Note'ni parse qilish
        const [, denomination, secretHex, nullifierHex] = noteString.split("-");
        const secret = Buffer.from(secretHex.slice(2), "hex");
        const nullifier = Buffer.from(nullifierHex.slice(2), "hex");

        // 2. Merkle proof olish (on-chain'dan)
        const { root, pathElements, pathIndices, leafIndex } =
            await this.getMerkleProof(secret, nullifier);

        // 3. ZK Proof yaratish
        const input = {
            // Yashirin
            secret: BigInt("0x" + secret.toString("hex")),
            nullifier: BigInt("0x" + nullifier.toString("hex")),
            pathElements: pathElements,
            pathIndices: pathIndices,
            // Oshkor
            root: root,
            nullifierHash: await this.getNullifierHash(nullifier),
            recipient: BigInt(recipient),
            fee: 0n
        };

        console.log("ZK proof yaratilmoqda...");
        const { proof, publicSignals } = await snarkjs.plonk.fullProve(
            input,
            "build/withdraw_js/withdraw.wasm",
            "circuit.zkey"
        );

        // 4. Withdraw tranzaksiya
        const calldata = await snarkjs.plonk.exportSolidityCallData(proof, publicSignals);
        const pool = new ethers.Contract(this.poolAddress, POOL_ABI, this.signer);

        const tx = await pool.withdraw(
            calldata,
            root,
            publicSignals[1], // nullifierHash
            recipient,
            ethers.constants.AddressZero, // relayer yo'q
            0 // fee yo'q
        );

        return tx;
    }
}

module.exports = XnetPrivacy;
```

---

## 5-QADAM: Test

```bash
# Local XNET node ishga tushirish
./target/release/xnet-node --dev

# Test
npx hardhat test --network xnet_testnet

# Test scenariy:
# 1. Alice 1 XNC deposit qiladi (commitment yaratadi)
# 2. 100 blok o'tadi
# 3. Bob ZK proof bilan 1 XNC oladi (Alice bilan bog'liq emas!)
# 4. Explorer'da: faqat 2 ta tranzaksiya, kim kimga — yo'q
```

---

## Denominationlar va Anonymity Set

```
Pool       │ 1 XNC  │ 10 XNC │ 100 XNC │ 1000 XNC
───────────┼────────┼────────┼─────────┼──────────
Min users  │ 5      │ 3      │ 2       │ 1
Ideal      │ 100+   │ 50+    │ 20+     │ 5+
```

Ko'p odam bir xil pooldan foydalansa — anonymity set katta bo'ladi.
Anonymity set katta bo'lsa — kim kimga yubordi bilish qiyinlashadi.

---

## Litsenziya

Tornado Cash core: MIT License
XNET adaptatsiyasi: GPL-3.0
