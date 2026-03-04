<div align="center">

<img src="https://img.shields.io/badge/XNET-Blockchain-blue?style=for-the-badge" />
<img src="https://img.shields.io/badge/Substrate-Framework-green?style=for-the-badge" />
<img src="https://img.shields.io/badge/EVM-Compatible-orange?style=for-the-badge" />
<img src="https://img.shields.io/badge/ink!-WASM-purple?style=for-the-badge" />
<img src="https://img.shields.io/badge/ZKP-Enabled-red?style=for-the-badge" />

# XNET — Next Generation Layer 1 Blockchain

**A high-performance, privacy-first Layer 1 blockchain built on Substrate with dual smart contract execution (EVM + WASM), Zero-Knowledge Proof support, and cross-chain interoperability via XCM.**

[Website](xnetcoin.org) · [Documentation](https://github.com/xnetcoin) · [Testnet Explorer](xnetcoin.org) · [Discord]() · [Twitter](xnethq)

</div>

---

## 🌐 Overview

XNET is a Layer 1 blockchain designed to bridge the gap between **Ethereum compatibility** and **next-generation privacy technology**. By combining a dual smart contract stack (EVM + ink!/WASM), native ZKP support, and Polkadot's XCM cross-chain messaging, XNET provides developers and users with a powerful, flexible, and future-proof platform.

Built on **Substrate**, XNET inherits battle-tested consensus mechanisms (BABE + GRANDPA), forkless runtime upgrades, and seamless interoperability with the broader Polkadot/Kusama ecosystem.

---

## ✨ Key Features

### 🔴 Dual Smart Contract Stack
- **EVM Compatible** — Deploy any Solidity contract without modification. Full Ethereum tooling support: MetaMask, Hardhat, Foundry, Remix.
- **ink! / WASM** — Native Rust-based smart contracts for maximum performance and safety. Built with `cargo contract`.

### 🔵 Zero-Knowledge Proofs (ZKP)
- Native ZKP support for **private transactions** and **confidential smart contracts**
- BN128 precompiles (BN128Add, BN128Mul, BN128Pairing) — enabling zk-SNARKs on-chain
- zkEVM-compatible design for future privacy DeFi applications

### 🟢 Cross-Chain Interoperability (XCM)
- Native **XCM** messaging protocol for seamless asset and data transfer
- Interoperable with the entire Polkadot/Kusama ecosystem
- Bridge-ready architecture for Ethereum ↔ XNET ↔ Polkadot

### 🟡 Tokenomics
- **53,000,000 XNET** hard supply cap — no inflation beyond block rewards
- Bitcoin-style halving every **21,038,400 blocks** (~4 years)
- Block reward starts at **1.117 XNET** per block
- Transparent, predictable supply schedule

### ⚪ NPoS Consensus
- **BABE** block production + **GRANDPA** finality
- Nominated Proof-of-Stake (NPoS) with Sequential Phragmén election
- 2.5%–10% annual staking rewards based on participation rate

---

## 💰 Tokenomics

| Parameter | Value |
|-----------|-------|
| **Token Symbol** | XNET |
| **Total Supply** | 53,000,000 XNET |
| **Genesis Premine** | 6,000,000 XNET |
| **Block Rewards** | ~47,000,000 XNET |
| **Block Reward** | 1.117 XNET/block |
| **Halving Interval** | 21,038,400 blocks (~4 years) |
| **Block Time** | 6 seconds |
| **Min Validator Bond** | 8,000 XNET |
| **Min Nominator Bond** | 1,000 XNET |
| **EVM Chain ID** | 2009 |
| **SS58 Prefix** | 888 |

### Supply Distribution

```
Total Supply: 53,000,000 XNET
├── Genesis Premine:  6,000,000 XNET (11.3%)
│   └── Locked for 2.5 years via on-chain vesting
└── Block Rewards:   47,000,000 XNET (88.7%)
    ├── Halving 1 (Years 1-4):  ~23,500,000 XNET
    ├── Halving 2 (Years 4-8):  ~11,750,000 XNET
    ├── Halving 3 (Years 8-12):  ~5,875,000 XNET
    └── Subsequent halvings...
```

### Fee Distribution

Every transaction fee on XNET is automatically split:

```
Transaction Fee
├── 60% → On-chain Treasury  (ecosystem development)
├── 15% → Grant Pool         (developer grants & bounties)
└── 25% → Block Validator    (network security)
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│                  XNET Runtime                    │
├──────────────────┬──────────────────────────────┤
│   Substrate Core │  BABE + GRANDPA Consensus     │
│                  │  NPoS Staking                 │
│                  │  On-chain Treasury            │
│                  │  Block Reward (Halving)       │
├──────────────────┼──────────────────────────────┤
│   Smart          │  EVM (Solidity)               │
│   Contracts      │  ink! / WASM (Rust)           │
│                  │  Unified Fee Model            │
├──────────────────┼──────────────────────────────┤
│   Privacy        │  ZKP — zk-SNARKs              │
│                  │  BN128 Precompiles            │
│                  │  Private Transactions         │
├──────────────────┼──────────────────────────────┤
│   Interop        │  XCM Cross-Chain Messaging    │
│                  │  Polkadot/Kusama Compatible   │
│                  │  Ethereum Bridge              │
└──────────────────┴──────────────────────────────┘
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install substrate dependencies (Ubuntu/Debian)
sudo apt install -y git clang curl libssl-dev llvm libudev-dev make protobuf-compiler
```

### Build

```bash
# Clone the repository
git clone https://github.com/xnetcoin/xnet
cd xnet

# Build release binary
cargo build --release

# Build WASM runtime only
cargo build --release -p xnet-node 
```

### Run a Local Development Node

```bash
# Start a single-node development chain (ephemeral, --dev wipes on restart)
./target/release/xnet-node --dev

# Start with persistent state
./target/release/xnet-node --dev --base-path /tmp/xnet-dev
```

### Connect

Once running, connect via:
- **Polkadot.js Apps**: https://polkadot.js.org/apps → Settings → Custom endpoint → `ws://127.0.0.1:9944`
- **MetaMask / EVM**: Add custom network → RPC `http://127.0.0.1:9933` → Chain ID `2009`

---

## 📡 Network Information

| Network | Status | RPC Endpoint | Explorer |
|---------|--------|--------------|----------|
| **Mainnet** | 🔜 Coming Soon | — | — |
| **Testnet** | 🟡 In Development | — | — |
| **Local Dev** | ✅ Available | `http://127.0.0.1:9933` | Local |

---

## 🛣️ Roadmap

### ✅ Phase 1 — Foundation (Completed)
- [x] Substrate runtime core
- [x] BABE + GRANDPA consensus
- [x] NPoS staking with Bitcoin-style halving
- [x] EVM compatibility (Frontier)
- [x] ink! / WASM smart contracts
- [x] On-chain treasury + grant pool
- [x] Fee distribution system

### 🔄 Phase 2 — Privacy & Interop (In Progress)
- [ ] Full ZKP pallet integration (zk-SNARKs)
- [ ] Private transaction support
- [ ] XCM cross-chain messaging
- [ ] Rococo testnet connection
- [ ] Public testnet launch

### 🔜 Phase 3 — Ecosystem (Q3-Q4 2025)
- [ ] Public mainnet launch
- [ ] Native DEX deployment
- [ ] Ethereum ↔ XNET bridge
- [ ] Developer grants program launch
- [ ] CEX listings

### 🔜 Phase 4 — Scale (2026)
- [ ] zkEVM private smart contracts
- [ ] Polkadot parachain slot
- [ ] On-chain governance (replace Sudo)
- [ ] Mobile wallet
- [ ] 100+ dApp ecosystem

---

## 🔧 Smart Contract Development

### Deploy a Solidity Contract (EVM)

```javascript
// hardhat.config.js
module.exports = {
  networks: {
    xnet: {
      url: "https://rpc.xnetcoin.org",
      chainId: 2009,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
}
```

```bash
npx hardhat deploy --network xnet
```

### Deploy an ink! Contract (WASM)

```bash
# Install cargo-contract
cargo install cargo-contract

# Create new ink! contract
cargo contract new my_contract
cd my_contract

# Build contract
cargo contract build

# Deploy via contracts UI or CLI
cargo contract instantiate --suri //Alice --args false
```

---

## 📊 Technical Specifications

| Parameter | Value |
|-----------|-------|
| **Consensus** | BABE (block production) + GRANDPA (finality) |
| **Block Time** | 6 seconds |
| **Block Size** | 5 MB max |
| **Gas Limit** | 75,000,000 per block |
| **Finality** | ~12 seconds (2 blocks) |
| **Staking APY** | 2.5% – 10% (variable) |
| **Epoch Duration** | 1 hour |
| **Era Duration** | 24 hours |
| **Unbonding Period** | 28 days |
| **EVM Chain ID** | 2009 |
| **Framework** | Substrate (Rust) |

---

## 🤝 Contributing

We welcome contributions from the community!

```bash
# Fork and clone
git clone https://github.com/xnetcoin/xnet
cd xnet

# Create feature branch
git checkout -b feature/your-feature

# Make changes, then test
cargo test

# Submit pull request
```

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

---

## 🏆 Grants Program

XNET allocates **15% of all transaction fees** to a dedicated grant pool for:
- dApp developers building on XNET
- Security auditors and researchers
- Community tools and infrastructure
- Educational content

Apply for a grant: [grants.xnetcoin.org](#)

---

## 📜 License

This project is licensed under the **Apache 2.0 License** — see [LICENSE](LICENSE) for details.

---

## 🔗 Links

| Resource | Link |
|----------|------|
| Website | [xnetcoin.org](#) |
| Documentation | [docs.xnetcoin.org](#) |
| Block Explorer | [explorer.xnetcoin.org](#) |
| Discord | [discord.gg/xnet](#) |
| Twitter/X | [@xnethq](#) |
| Telegram | [t.me/xnethq](#) |

---

<div align="center">

**Built with ❤️ using Substrate & Rust**

*XNET — Where Privacy Meets Performance*

</div>