// scripts/deploy.js
//
// XNET Privacy Pool to'liq deploy scripti.
//
// Ishlatish:
//   npx hardhat run scripts/deploy.js --network xnet_local
//   npx hardhat run scripts/deploy.js --network xnet_testnet
//
// Natija: deployed_addresses.json faylida contract manzillari saqlanadi.

const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

// ─────────────────────────────────────────────────────────────────────────────
// Denominationlar (XNC)
// Har biri uchun alohida pool — anonymity set ajratilgan
// ─────────────────────────────────────────────────────────────────────────────
const DENOMINATIONS = [
    ethers.parseEther("0.1"),    //   0.1 XNC
    ethers.parseEther("1"),      //   1   XNC
    ethers.parseEther("10"),     //  10   XNC
    ethers.parseEther("100"),    // 100   XNC
];

const DENOMINATION_NAMES = ["0.1_XNC", "1_XNC", "10_XNC", "100_XNC"];

// ─────────────────────────────────────────────────────────────────────────────
async function main() {
    const [deployer] = await ethers.getSigners();
    const network = await ethers.provider.getNetwork();

    console.log("═══════════════════════════════════════════════");
    console.log("  XNET Privacy Pool — Deploy");
    console.log("═══════════════════════════════════════════════");
    console.log(`  Network  : ${network.name} (chainId: ${network.chainId})`);
    console.log(`  Deployer : ${deployer.address}`);
    console.log(`  Balance  : ${ethers.formatEther(
        await ethers.provider.getBalance(deployer.address)
    )} XNC`);
    console.log("═══════════════════════════════════════════════\n");

    const deployed = {
        network: network.name,
        chainId: network.chainId.toString(),
        deployer: deployer.address,
        timestamp: new Date().toISOString(),
        contracts: {}
    };

    // ── 1. Verifier deploy ─────────────────────────────────────────────────
    // snarkjs tomonidan generatsiya qilingan PlonkVerifier
    // (snarkjs zkey export solidityverifier circuit.zkey contracts/Verifier.sol)
    console.log("1. PlonkVerifier deploy qilinmoqda...");
    const Verifier = await ethers.getContractFactory("PlonkVerifier");
    const verifier = await Verifier.deploy();
    await verifier.waitForDeployment();
    deployed.contracts.verifier = await verifier.getAddress();
    console.log(`   ✅ Verifier: ${deployed.contracts.verifier}\n`);

    // ── 2. Poseidon Hasher deploy ──────────────────────────────────────────
    // ZK-friendly hash funksiyasi — Merkle tree uchun
    console.log("2. PoseidonHasher deploy qilinmoqda...");
    const Hasher = await ethers.getContractFactory("PoseidonHasher");
    const hasher = await Hasher.deploy();
    await hasher.waitForDeployment();
    deployed.contracts.hasher = await hasher.getAddress();
    console.log(`   ✅ Hasher: ${deployed.contracts.hasher}\n`);

    // ── 3. Privacy Pool'lar deploy ─────────────────────────────────────────
    console.log("3. Privacy Pool'lar deploy qilinmoqda...");
    deployed.contracts.pools = {};

    const Pool = await ethers.getContractFactory("contracts/XnetPrivacyPool.sol:XnetPrivacyPool");

    for (let i = 0; i < DENOMINATIONS.length; i++) {
        const denomination = DENOMINATIONS[i];
        const name = DENOMINATION_NAMES[i];

        process.stdout.write(`   ${name} pool...`);

        const pool = await Pool.deploy(
            deployed.contracts.verifier,
            deployed.contracts.hasher,
            denomination,
            deployer.address   // operator (relayer to'lovini oladi)
        );
        await pool.waitForDeployment();

        const poolAddress = await pool.getAddress();
        deployed.contracts.pools[name] = poolAddress;

        console.log(` ✅ ${poolAddress}`);
    }

    // ── 4. Factory deploy (ixtiyoriy) ─────────────────────────────────────
    console.log("\n4. XnetPrivacyPoolFactory deploy qilinmoqda...");
    const Factory = await ethers.getContractFactory("contracts/XnetPrivacyPool.sol:XnetPrivacyPoolFactory");
    const factory = await Factory.deploy(
        deployed.contracts.verifier,
        deployed.contracts.hasher
    );
    await factory.waitForDeployment();
    deployed.contracts.factory = await factory.getAddress();
    console.log(`   ✅ Factory: ${deployed.contracts.factory}\n`);

    // ── 5. Natijalarni saqlash ─────────────────────────────────────────────
    const outputPath = path.join(__dirname, "../deployed_addresses.json");
    fs.writeFileSync(outputPath, JSON.stringify(deployed, null, 2));

    console.log("═══════════════════════════════════════════════");
    console.log("  Deploy muvaffaqiyatli yakunlandi!");
    console.log("═══════════════════════════════════════════════");
    console.log(`  Verifier : ${deployed.contracts.verifier}`);
    console.log(`  Hasher   : ${deployed.contracts.hasher}`);
    console.log(`  Factory  : ${deployed.contracts.factory}`);
    console.log("\n  Poollar:");
    for (const [name, addr] of Object.entries(deployed.contracts.pools)) {
        console.log(`    ${name.padEnd(12)} : ${addr}`);
    }
    console.log(`\n  Manzillar saqlandi: deployed_addresses.json`);
    console.log("═══════════════════════════════════════════════");
}

main().catch((err) => {
    console.error("Deploy xatosi:", err);
    process.exit(1);
});