// sdk/xnet-privacy-sdk.js
//
// XNET Privacy Pool — JavaScript SDK
//
// O'rnatish:
//   npm install ethers circomlibjs snarkjs
//
// Ishlatish:
//   const XnetPrivacy = require("./sdk/xnet-privacy-sdk");
//   const sdk = new XnetPrivacy({ rpc: "http://127.0.0.1:8545" });
//   const note = await sdk.deposit(signer, poolAddress, "1");
//   await sdk.withdraw(signer, note, recipientAddress);

"use strict";

const { ethers } = require("ethers");
const { buildPoseidon } = require("circomlibjs");
const snarkjs = require("snarkjs");
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

// ─────────────────────────────────────────────────────────────────────────────
// Pool ABI — faqat kerakli funksiyalar
// ─────────────────────────────────────────────────────────────────────────────
const POOL_ABI = [
    "function deposit(bytes32 commitment) external payable",
    "function withdraw(uint[2] a, uint[2][2] b, uint[2] c, bytes32 root, bytes32 nullifierHash, address recipient, address relayer, uint256 fee) external",
    "function isKnownRoot(bytes32 root) external view returns (bool)",
    "function getLastRoot() external view returns (bytes32)",
    "function commitments(bytes32) external view returns (bool)",
    "function nullifierHashes(bytes32) external view returns (bool)",
    "function denomination() external view returns (uint256)",
    "function nextIndex() external view returns (uint32)",
    "event Deposit(bytes32 indexed commitment, uint32 leafIndex, uint256 timestamp)",
    "event Withdrawal(address to, bytes32 nullifierHash, address indexed relayer, uint256 fee)",
];

// ─────────────────────────────────────────────────────────────────────────────
// XnetPrivacy SDK
// ─────────────────────────────────────────────────────────────────────────────
class XnetPrivacy {
    /**
     * @param {object} config
     * @param {string} config.rpc      — XNET RPC URL
     * @param {string} [config.wasmPath]  — withdraw.wasm yo'li
     * @param {string} [config.zkeyPath]  — circuit.zkey yo'li
     */
    constructor(config = {}) {
        this.rpc = config.rpc || "http://127.0.0.1:8545";
        this.provider = new ethers.JsonRpcProvider(this.rpc);
        this.wasmPath = config.wasmPath || "./build/withdraw_js/withdraw.wasm";
        this.zkeyPath = config.zkeyPath || "./circuit.zkey";
        this._poseidon = null;
    }

    // ── Poseidon hasher (lazy init) ──────────────────────────────────────────
    async _getHasher() {
        if (!this._poseidon) {
            this._poseidon = await buildPoseidon();
        }
        return this._poseidon;
    }

    // ── Commitment hisoblash ─────────────────────────────────────────────────
    async _computeCommitment(secret, nullifier) {
        const poseidon = await this._getHasher();
        const hash = poseidon([
            BigInt("0x" + secret.toString("hex")),
            BigInt("0x" + nullifier.toString("hex")),
        ]);
        return "0x" + poseidon.F.toString(hash, 16).padStart(64, "0");
    }

    // ── Nullifier hash hisoblash ─────────────────────────────────────────────
    async _computeNullifierHash(nullifier) {
        const poseidon = await this._getHasher();
        const hash = poseidon([BigInt("0x" + nullifier.toString("hex"))]);
        return "0x" + poseidon.F.toString(hash, 16).padStart(64, "0");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DEPOSIT
    // ─────────────────────────────────────────────────────────────────────────

    /**
     * XNC'ni privacy pool'ga yashirish.
     *
     * @param {ethers.Signer} signer       — jo'natuvchi
     * @param {string}        poolAddress  — pool contract manzili
     * @param {string}        amount       — "1", "10", "100" (XNC)
     * @returns {object} note — xavfsiz saqlash kerak!
     *
     * @example
     * const note = await sdk.deposit(signer, "0x1234...", "1");
     * console.log("NOTE SAQLANG:", note.noteString);
     */
    async deposit(signer, poolAddress, amount) {
        console.log(`\n🔒 Deposit: ${amount} XNC → Privacy Pool`);

        // 1. Random secret va nullifier
        const secret = crypto.randomBytes(31);
        const nullifier = crypto.randomBytes(31);

        console.log("   Secret va nullifier yaratildi (yashirin)");

        // 2. Commitment = Poseidon(secret, nullifier)
        const commitment = await this._computeCommitment(secret, nullifier);
        console.log(`   Commitment: ${commitment.slice(0, 20)}...`);

        // 3. Pool contract
        const pool = new ethers.Contract(poolAddress, POOL_ABI, signer);

        // 4. Denomination tekshirish
        const denomination = await pool.denomination();
        const depositAmount = ethers.parseEther(amount);
        if (depositAmount !== denomination) {
            throw new Error(
                `Noto'g'ri miqdor. Pool denomination: ${ethers.formatEther(denomination)} XNC`
            );
        }

        // 5. Tranzaksiya yuborish
        console.log("   Tranzaksiya yuborilmoqda...");
        const tx = await pool.deposit(commitment, { value: depositAmount });
        const receipt = await tx.wait();

        // 6. Leaf index olish (Deposit event'dan)
        const depositEvent = receipt.logs
            .map(log => { try { return pool.interface.parseLog(log); } catch { return null; } })
            .find(e => e && e.name === "Deposit");

        const leafIndex = depositEvent ? Number(depositEvent.args.leafIndex) : 0;
        console.log(`   ✅ Deposit muvaffaqiyatli! Leaf index: ${leafIndex}`);

        // 7. Note — foydalanuvchi saqlashi shart!
        const note = {
            noteString: `xnet-${amount}xnc-${secret.toString("hex")}-${nullifier.toString("hex")}`,
            secret: secret.toString("hex"),
            nullifier: nullifier.toString("hex"),
            commitment,
            nullifierHash: await this._computeNullifierHash(nullifier),
            poolAddress,
            amount,
            leafIndex,
            txHash: receipt.hash,
            timestamp: Date.now(),
            network: (await this.provider.getNetwork()).chainId.toString(),
        };

        console.log("\n⚠️  NOTE'NI XAVFSIZ SAQLANG!");
        console.log(`   ${note.noteString}`);
        console.log("   Bu note yo'qolsa — XNC'ingiz qaytarib bo'lmaydi!\n");

        return note;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // WITHDRAW
    // ─────────────────────────────────────────────────────────────────────────

    /**
     * Note bilan XNC'ni istalgan addressga chiqarish.
     *
     * @param {ethers.Signer} signer       — istalgan signer (depositor emas!)
     * @param {string|object} note         — noteString yoki note object
     * @param {string}        recipient    — XNC qabul qiluvchi address
     * @param {object}        [options]
     * @param {string}        [options.relayer]  — relayer address (gasless uchun)
     * @param {string}        [options.fee]      — relayer to'lovi (XNC)
     *
     * @example
     * await sdk.withdraw(
     *   signer,
     *   "xnet-1xnc-abc123...-def456...",
     *   "0x742d35Cc6634C0532925a3b8D4C9C..."
     * );
     */
    async withdraw(signer, note, recipient, options = {}) {
        // Note parse
        const parsedNote = typeof note === "string"
            ? this._parseNoteString(note)
            : note;

        console.log(`\n🔓 Withdraw: ${parsedNote.amount} XNC → ${recipient.slice(0, 10)}...`);

        const pool = new ethers.Contract(parsedNote.poolAddress, POOL_ABI, signer);

        // 1. Nullifier sarflanmaganmi?
        const nullifierHash = parsedNote.nullifierHash ||
            await this._computeNullifierHash(Buffer.from(parsedNote.nullifier, "hex"));

        const isSpent = await pool.nullifierHashes(nullifierHash);
        if (isSpent) throw new Error("Bu note allaqachon sarflangan!");

        // 2. Merkle proof yaratish
        console.log("   Merkle proof yaratilmoqda...");
        const { root, pathElements, pathIndices } =
            await this._getMerkleProof(pool, parsedNote);

        const rootIsKnown = await pool.isKnownRoot(root);
        if (!rootIsKnown) throw new Error("Merkle root noma'lum!");

        // 3. ZK Proof yaratish (eng uzoq qadam)
        console.log("   ZK proof yaratilmoqda (30-60 soniya)...");

        const input = {
            // Yashirin
            secret:       BigInt("0x" + parsedNote.secret),
            nullifier:    BigInt("0x" + parsedNote.nullifier),
            pathElements: pathElements.map(e => BigInt(e)),
            pathIndices:  pathIndices,
            // Oshkor
            root:         BigInt(root),
            nullifierHash: BigInt(nullifierHash),
            recipient:    BigInt(recipient),
            fee:          BigInt(ethers.parseEther(options.fee || "0")),
        };

        const { proof, publicSignals } = await snarkjs.plonk.fullProve(
            input,
            this.wasmPath,
            this.zkeyPath
        );

        console.log("   ✅ ZK proof yaratildi!");

        // 4. Calldata tayyorlash
        const calldataStr = await snarkjs.plonk.exportSolidityCallData(
            proof, publicSignals
        );
        const calldata = JSON.parse(`[${calldataStr}]`);
        const [proofA, proofB, proofC] = calldata;

        // 5. Withdraw tranzaksiya
        console.log("   Withdraw tranzaksiyasi yuborilmoqda...");

        const relayerAddress = options.relayer || ethers.ZeroAddress;
        const feeAmount = ethers.parseEther(options.fee || "0");

        const tx = await pool.withdraw(
            proofA, proofB, proofC,
            root,
            nullifierHash,
            recipient,
            relayerAddress,
            feeAmount
        );

        const receipt = await tx.wait();
        console.log(`   ✅ Withdraw muvaffaqiyatli!`);
        console.log(`   Tx: ${receipt.hash}`);
        console.log(`   Recipient: ${recipient}`);
        console.log(`   Miqdor: ${parsedNote.amount} XNC\n`);

        return receipt;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Yordamchi funksiyalar
    // ─────────────────────────────────────────────────────────────────────────

    /**
     * Note string'ni parse qilish.
     * Format: xnet-{amount}xnc-{secret_hex}-{nullifier_hex}
     */
    _parseNoteString(noteString) {
        const parts = noteString.split("-");
        if (parts.length < 4 || parts[0] !== "xnet") {
            throw new Error("Noto'g'ri note format! xnet-{amount}xnc-{secret}-{nullifier}");
        }
        const amount = parts[1].replace("xnc", "");
        const secret = parts[2];
        const nullifier = parts[3];

        // Pool manzili deployed_addresses.json dan
        let poolAddress;
        try {
            const deployed = JSON.parse(
                fs.readFileSync("./deployed_addresses.json", "utf8")
            );
            poolAddress = deployed.contracts.pools[`${amount}_XNC`];
        } catch {
            throw new Error("deployed_addresses.json topilmadi. Avval deploy qiling.");
        }

        return { amount, secret, nullifier, poolAddress };
    }

    /**
     * Merkle proof olish (on-chain event'lardan).
     *
     * Deposit event'larini o'qib, commitment'ning Merkle path'ini hisoblaydi.
     */
    async _getMerkleProof(pool, note) {
        const commitment = await this._computeCommitment(
            Buffer.from(note.secret, "hex"),
            Buffer.from(note.nullifier, "hex")
        );

        // Barcha Deposit event'larini olish
        const filter = pool.filters.Deposit();
        const events = await pool.queryFilter(filter, 0, "latest");

        // Commitment'ni topish
        const leaves = events.map(e => e.args.commitment);
        const leafIndex = leaves.findIndex(l => l === commitment);

        if (leafIndex === -1) {
            throw new Error("Commitment topilmadi! Note noto'g'ri yoki deposit bo'lmagan.");
        }

        // Merkle path hisoblash
        const { pathElements, pathIndices } =
            await this._computeMerklePath(leaves, leafIndex);

        const root = await pool.getLastRoot();

        return { root, pathElements, pathIndices, leafIndex };
    }

    /**
     * Merkle path hisoblash (20 daraja).
     */
    async _computeMerklePath(leaves, targetIndex) {
        const poseidon = await this._getHasher();
        const TREE_HEIGHT = 20;

        // Bo'sh barg qiymati
        const ZERO = "0x0000000000000000000000000000000000000000000000000000000000000000";

        // Tree'ni to'ldirish
        let currentLevel = [...leaves];
        while (currentLevel.length < 2 ** TREE_HEIGHT) {
            currentLevel.push(ZERO);
        }

        const pathElements = [];
        const pathIndices = [];
        let currentIndex = targetIndex;

        for (let level = 0; level < TREE_HEIGHT; level++) {
            const isRight = currentIndex % 2 === 1;
            const siblingIndex = isRight ? currentIndex - 1 : currentIndex + 1;

            pathElements.push(
                siblingIndex < currentLevel.length ? currentLevel[siblingIndex] : ZERO
            );
            pathIndices.push(isRight ? 1 : 0);

            // Yuqori darajani hisoblash
            const nextLevel = [];
            for (let i = 0; i < currentLevel.length; i += 2) {
                const left = currentLevel[i];
                const right = currentLevel[i + 1] || ZERO;
                const hash = poseidon([BigInt(left), BigInt(right)]);
                nextLevel.push("0x" + poseidon.F.toString(hash, 16).padStart(64, "0"));
            }
            currentLevel = nextLevel;
            currentIndex = Math.floor(currentIndex / 2);
        }

        return { pathElements, pathIndices };
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Status tekshirish
    // ─────────────────────────────────────────────────────────────────────────

    /**
     * Note holatini tekshirish.
     */
    async checkNote(noteString) {
        const note = this._parseNoteString(noteString);
        const pool = new ethers.Contract(note.poolAddress, POOL_ABI, this.provider);

        const commitment = await this._computeCommitment(
            Buffer.from(note.secret, "hex"),
            Buffer.from(note.nullifier, "hex")
        );
        const nullifierHash = await this._computeNullifierHash(
            Buffer.from(note.nullifier, "hex")
        );

        const isDeposited = await pool.commitments(commitment);
        const isSpent = await pool.nullifierHashes(nullifierHash);

        return {
            isDeposited,
            isSpent,
            status: !isDeposited ? "⚠️  Deposit topilmadi"
                : isSpent ? "✅ Sarflangan (withdraw qilingan)"
                : "🔒 Faol (withdraw qilish mumkin)",
        };
    }

    /**
     * Pool statistikasi.
     */
    async getPoolStats(poolAddress) {
        const pool = new ethers.Contract(poolAddress, POOL_ABI, this.provider);
        const [denomination, nextIndex, lastRoot] = await Promise.all([
            pool.denomination(),
            pool.nextIndex(),
            pool.getLastRoot(),
        ]);

        return {
            denomination: ethers.formatEther(denomination) + " XNC",
            totalDeposits: Number(nextIndex),
            lastRoot: lastRoot.slice(0, 20) + "...",
            anonymitySet: Number(nextIndex),
        };
    }
}

module.exports = XnetPrivacy;

// ─────────────────────────────────────────────────────────────────────────────
// CLI — to'g'ridan-to'g'ri ishga tushirish
// node sdk/xnet-privacy-sdk.js deposit  <pool_address> <amount>
// node sdk/xnet-privacy-sdk.js withdraw <note_string> <recipient>
// node sdk/xnet-privacy-sdk.js check    <note_string>
// ─────────────────────────────────────────────────────────────────────────────
if (require.main === module) {
    const [,, command, ...args] = process.argv;

    (async () => {
        const sdk = new XnetPrivacy({ rpc: "http://127.0.0.1:8545" });
        const provider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");

        // Test signer (local dev)
        const signer = new ethers.Wallet(
            "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133",
            provider
        );

        switch (command) {
            case "deposit": {
                const [poolAddress, amount] = args;
                if (!poolAddress || !amount) {
                    console.log("Ishlatish: node sdk/xnet-privacy-sdk.js deposit <pool_address> <amount>");
                    break;
                }
                const note = await sdk.deposit(signer, poolAddress, amount);
                console.log("\nNote saqlandi: note.json");
                fs.writeFileSync("note.json", JSON.stringify(note, null, 2));
                break;
            }

            case "withdraw": {
                const [noteString, recipient] = args;
                if (!noteString || !recipient) {
                    console.log("Ishlatish: node sdk/xnet-privacy-sdk.js withdraw <note_string> <recipient>");
                    break;
                }
                await sdk.withdraw(signer, noteString, recipient);
                break;
            }

            case "check": {
                const [noteString] = args;
                if (!noteString) {
                    console.log("Ishlatish: node sdk/xnet-privacy-sdk.js check <note_string>");
                    break;
                }
                const status = await sdk.checkNote(noteString);
                console.log("\nNote holati:", status.status);
                console.log("Deposit:", status.isDeposited);
                console.log("Sarflangan:", status.isSpent);
                break;
            }

            default:
                console.log("Buyruqlar: deposit | withdraw | check");
        }
    })().catch(console.error);
}