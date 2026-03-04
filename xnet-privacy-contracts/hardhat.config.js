require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config();

// ─────────────────────────────────────────────────────────────────────────────
// XNET Hardhat konfiguratsiyasi
//
// O'rnatish:
//   npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox dotenv
//
// .env fayl yarating:
//   DEPLOYER_PRIVATE_KEY=0x... (test kaliti)
//   XNET_TESTNET_RPC=http://127.0.0.1:8545
//   XNET_MAINNET_RPC=https://rpc.xnetcoin.org
// ─────────────────────────────────────────────────────────────────────────────

const DEPLOYER_KEY = process.env.DEPLOYER_PRIVATE_KEY ||
    "0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391";

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
    solidity: {
        version: "0.8.20",
        settings: {
            optimizer: {
                enabled: true,
                runs: 200,
            },
        },
    },

    networks: {
        // ── Local development ──────────────────────────────────────────────
        hardhat: {
            chainId: 31337,
        },

        // ── XNET Local node (--dev rejimida) ──────────────────────────────
        xnet_local: {
            url: "http://127.0.0.1:9944",
            chainId: 2009,
            accounts: [DEPLOYER_KEY],
            // Fixed gas avoids eth_estimateGas → BlockBuilder_inherent_extrinsics
            // which panics with pallet_timestamp in BABE --dev mode.
            gas: 10_000_000,
            gasPrice: 1_000_000_000, // 1 Gwei
            // gasMultiplier: 1 prevents Hardhat multiplying the fixed gas value
            // (it does NOT skip estimation — use `gas` as a fixed limit instead).
            timeout: 120_000,
        },

        // ── XNET Testnet ───────────────────────────────────────────────────
        xnet_testnet: {
            url: process.env.XNET_TESTNET_RPC || "https://testnet-rpc.xnetcoin.org",
            chainId: 2009,
            accounts: [DEPLOYER_KEY],
            gas: 5_000_000,
            gasPrice: 1_000_000_000,
            timeout: 60_000,
        },

        // ── XNET Mainnet ───────────────────────────────────────────────────
        xnet_mainnet: {
            url: process.env.XNET_MAINNET_RPC || "https://rpc.xnetcoin.org",
            chainId: 2009,
            accounts: process.env.MAINNET_PRIVATE_KEY
                ? [process.env.MAINNET_PRIVATE_KEY]
                : [], // Mainnet uchun alohida kalit
            gas: 5_000_000,
            gasPrice: 1_000_000_000,
            timeout: 120_000,
        },
    },

    // Contract manzillarini saqlash
    paths: {
        sources: "./contracts",
        tests: "./test",
        cache: "./cache",
        artifacts: "./artifacts",
    },

    // Gas hisoboti
    gasReporter: {
        enabled: process.env.REPORT_GAS === "true",
        currency: "USD",
        token: "XNC",
    },
};