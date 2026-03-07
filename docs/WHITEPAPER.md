# XNET Blockchain — Technical Whitepaper v1.0

**Version 1.0 · 2025**

🌐 xnetcoin.org ·   t.me/xnethq · X @xnethq

---

## Abstract

XNET is a high-performance, privacy-first Layer 1 blockchain built on the Substrate framework. By combining EVM (Solidity) and ink!/WASM (Rust) dual smart contracts with native Zero-Knowledge Proof support and XCM cross-chain interoperability, XNET delivers compatibility, privacy, performance, and interoperability — simultaneously.

Hard supply cap: **53,000,000 XNET**. Bitcoin-style halving. Transparent fee model.

> **Core Thesis:** The next billion blockchain users will not come from chains that force a choice between privacy, compatibility, and performance. XNET removes that trade-off entirely.

---

## Table of Contents

1. [Introduction & Problem Statement](#1-introduction--problem-statement)
2. [Solution: The XNET Architecture](#2-solution-the-xnet-architecture)
3. [Consensus Mechanism](#3-consensus-mechanism)
4. [Smart Contract Platform](#4-smart-contract-platform)
5. [Zero-Knowledge Proof Layer](#5-zero-knowledge-proof-layer)
6. [Cross-Chain Interoperability (XCM)](#6-cross-chain-interoperability-xcm)
7. [Tokenomics](#7-tokenomics)
8. [Network Economics & Fee Model](#8-network-economics--fee-model)
9. [Roadmap](#9-roadmap)
10. [Security](#10-security)
11. [Conclusion](#11-conclusion)

---

## 1. Introduction & Problem Statement

Blockchain technology has evolved rapidly, yet the ecosystem remains fragmented along three critical dimensions:

**Compatibility vs. Privacy**
Ethereum and EVM-compatible chains offer broad developer tooling but provide no native privacy. Every transaction is fully public — a fundamental limitation for real-world adoption.

**Performance vs. Decentralization**
High-throughput chains often sacrifice decentralization. XNET achieves competitive throughput through WASM execution and optimized consensus without compromising validator distribution.

**Isolation vs. Interoperability**
Most chains operate in isolation. Assets and data cannot move freely between ecosystems, creating value silos and fragmented liquidity.

XNET addresses all three problems simultaneously.

---

## 2. Solution: The XNET Architecture

XNET is built on **Substrate**, the battle-tested blockchain framework used as the foundation of the Polkadot ecosystem. Substrate provides forkless runtime upgrades, modular pallet-based architecture, and native WASM compilation.

### 2.1 Core Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Consensus | BABE + GRANDPA | Block production + deterministic finality |
| Execution | EVM + ink!/WASM | Dual smart contract environments |
| Privacy | ZKP / zk-SNARKs | Private transactions & confidential contracts |
| Interop | XCM Protocol | Cross-chain messaging & asset transfer |
| Staking | NPoS (Phragmén) | Decentralized validator selection |
| Framework | Substrate (Rust) | Modular, upgradeable runtime |

### 2.2 Key Design Principles

- **Developer First** — MetaMask, Hardhat, Foundry, Remix work out of the box
- **Privacy by Default** — ZKP is built into the runtime, not added later
- **Sound Money** — Hard cap + halving = predictable, deflationary supply
- **Forkless Upgrades** — Runtime upgrades deployed on-chain, no hard forks
- **Ecosystem Aligned** — Fees automatically fund treasury, grants, and validators

---

## 3. Consensus Mechanism

### 3.1 BABE — Block Production

XNET uses **BABE** (Blind Assignment for Blockchain Extension) with VRF-based slot assignment, preventing adversaries from predicting future block authors.

| Parameter | Value |
|-----------|-------|
| Block Time | 6 seconds |
| Epoch Duration | 1 hour (600 blocks) |
| Slot Assignment | VRF-based (unpredictable) |
| Max Authorities | 1,000 validators |

### 3.2 GRANDPA — Finality

**GRANDPA** provides deterministic Byzantine fault-tolerant finality. Once a block is finalized, it cannot be reverted as long as fewer than 1/3 of validators are malicious. Target finality: **12 seconds**.

### 3.3 NPoS — Validator Selection

Validators are selected via Nominated Proof-of-Stake using the **Sequential Phragmén** algorithm — optimizing for proportional representation.

| Parameter | Value |
|-----------|-------|
| Minimum Validator Bond | 8,000 XNET |
| Minimum Nominator Bond | 1,000 XNET |
| Active Validator Slots | Up to 100 |
| Era Duration | 24 hours |
| Unbonding Period | 28 days |
| Slash Deferral | 7 days |

---

## 4. Smart Contract Platform

### 4.1 EVM — Ethereum Virtual Machine

Full Ethereum compatibility via **Frontier**. Any Solidity contract deploys without modification.

- Chain ID: `2009`
- Gas Limit: 75,000,000 per block
- Fee Model: EIP-1559 dynamic base fee (starts at 1 Gwei)
- Precompiles: Full set (0x01–0x09) including BN128 for zk-SNARKs
- Tooling: MetaMask, Hardhat, Foundry, Remix, Ethers.js

### 4.2 ink! — WASM Smart Contracts

Rust-based contracts compiling to WebAssembly. Stronger type safety, no integer overflow, smaller binaries, superior performance.

- Language: Rust (ink! macros)
- Execution: WebAssembly — fast, deterministic, sandboxed
- Storage: Pay-per-item — encourages efficient state management
- Tooling: `cargo-contract`, Contracts UI

> Both EVM and ink! share the same XNET token and fee model. Existing Ethereum projects migrate with zero code changes.

---

## 5. Zero-Knowledge Proof Layer

### 5.1 ZKP Infrastructure

XNET's ZKP layer is built around **zk-SNARKs** using the BN128 elliptic curve — the same used by Ethereum's zkEVM and Zcash. Full ZKP pallet integration is currently in active development.

### 5.2 EVM Precompiles for ZKP

| Precompile | Address | Function |
|-----------|---------|---------|
| BN128Add | `0x06` | Elliptic curve point addition |
| BN128Mul | `0x07` | Scalar multiplication |
| BN128Pairing | `0x08` | Pairing checks (proof verification) |
| BLAKE2F | `0x09` | BLAKE2 hash (Zcash compatibility) |

### 5.3 Privacy Use Cases

- **Private Transactions** — Hide sender, receiver, and amount
- **Confidential Contracts** — Execute logic with private inputs
- **Anonymous Governance** — Verifiable but anonymous on-chain voting
- **Private DeFi** — Trade without exposing positions to front-runners
- **Identity & KYC** — Prove attributes without revealing raw data

---

## 6. Cross-Chain Interoperability (XCM)

XNET implements **XCM** (Cross-Consensus Messaging) — Polkadot's native trustless cross-chain protocol.

- **Asset Transfer** — Move XNET to/from Polkadot parachains
- **Remote Execution** — Trigger contract calls on other chains
- **Cross-Chain DeFi** — Liquidity pools spanning multiple chains
- **Ethereum Bridge** — XCM + Ethereum light client verification

> Currently integrating with Rococo testnet. Full Polkadot ecosystem integration targeted for Phase 3.

---

## 7. Tokenomics

### 7.1 Supply Overview

| Parameter | Value |
|-----------|-------|
| Token Symbol | XNET |
| Decimals | 18 |
| Hard Supply Cap | 53,000,000 XNET |
| Genesis Premine | 6,000,000 XNET (11.3%) |
| Block Rewards | ~47,000,000 XNET (88.7%) |
| Initial Block Reward | 1.117 XNET per block |
| Block Time | 6 seconds |
| Halving Interval | 21,038,400 blocks (~4 years) |
| EVM Chain ID | 2009 |
| SS58 Prefix | 888 |

### 7.2 Genesis Premine — 6,000,000 XNET

| Allocation | Amount | Purpose | Vesting |
|-----------|--------|---------|---------|
| Founder & Team | 1,000,000 XNET | Core development | 2.5 years linear |
| Investor Reserve | 1,000,000 XNET | Seed & strategic investors | 2.5 years linear |
| Ecosystem Fund | 2,000,000 XNET | Grants, partnerships, dApps | 3 years |
| Treasury Reserve | 2,000,000 XNET | Ops, audits, legal | On-chain governance |

> 🔒 **All premine tokens are locked for minimum 2.5 years via `pallet-vesting`. Enforced at protocol level — no exceptions.**

### 7.3 Halving Schedule

| Period | Reward/Block | Total Minted | Cumulative |
|--------|-------------|-------------|-----------|
| Genesis | — | 6,000,000 | 6,000,000 |
| Year 0–4 | 1.117 XNET | 23,499,892 | 29,499,892 |
| Year 4–8 | 0.5585 XNET | 11,749,946 | 41,249,838 |
| Year 8–12 | 0.27925 XNET | 5,874,973 | 47,124,811 |
| Year 12+ | 0.13962 XNET | ~5,875,189 | ~53,000,000 |

---

## 8. Network Economics & Fee Model

### 8.1 Fee Distribution

| Recipient | Share | Purpose |
|-----------|-------|---------|
| On-chain Treasury | **60%** | Development, audits, operations |
| Developer Grant Pool | **15%** | Grants for XNET builders |
| Block Validator | **25%** | Block production incentive |

### 8.2 Staking Rewards

- Minimum APY: **2.5%**
- Maximum APY: **10%** (at 50% staking ratio)
- Target staking ratio: **50%** of circulating supply

### 8.3 Developer Grants

15% of all fees flow into a dedicated on-chain grant pool — a self-sustaining mechanism that grows with network usage.

---

## 9. Roadmap

### ✅ Phase 1 — Foundation (Completed)
- [x] Substrate runtime: BABE + GRANDPA + NPoS
- [x] Bitcoin-style halving (53M hard cap)
- [x] EVM via Frontier (Chain ID 2009)
- [x] ink!/WASM smart contracts
- [x] 60/15/25 fee distribution
- [x] On-chain treasury + grant pool
- [x] 2.5-year vesting lock

### 🔄 Phase 2 — Privacy & Interop (In Progress)
- [ ] Full ZKP pallet (zk-SNARKs)
- [ ] Private transactions
- [ ] XCM implementation
- [ ] Rococo testnet connection
- [ ] Public testnet + faucet
- [ ] Developer docs portal

### 🔜 Phase 3 — Ecosystem (Q3–Q4 2025)
- [ ] Mainnet launch
- [ ] Native DEX
- [ ] Ethereum ↔ XNET bridge
- [ ] Grants program
- [ ] CEX listings (MEXC, Gate.io)
- [ ] First dApp ecosystem

### 🔜 Phase 4 — Scale (2026)
- [ ] zkEVM private contracts
- [ ] Polkadot parachain slot
- [ ] On-chain governance
- [ ] Mobile wallet
- [ ] 100+ dApps
- [ ] Top 100 listings

---

## 10. Security

### 10.1 Consensus
GRANDPA: deterministic finality, BFT up to 1/3 malicious validators. BABE: VRF slot assignment prevents prediction attacks.

### 10.2 Smart Contracts
`CallFilter: Nothing` blocks contracts from accessing Sudo, Staking, Governance. ink!/WASM eliminates Solidity vulnerability classes at the language level.

### 10.3 Slashing
7-day deferral on all slashes — governance can review and cancel erroneous penalties before execution.

### 10.4 Audits
Independent security audits of runtime, EVM, ZKP, and contract pallets scheduled before mainnet launch.

---

## 11. Conclusion

XNET does not force developers or users to choose between compatibility and privacy, performance and decentralization, or isolation and interoperability.

The **53M hard cap**, halving schedule, transparent fee model, and **2.5-year vesting lock** on all premine tokens demonstrate a long-term commitment to building a sustainable, trustworthy ecosystem.

> **XNET is not just another EVM chain. It is the privacy and performance layer the Polkadot ecosystem has been waiting for.**

---

## Links

| Resource | Link |
|----------|------|
| Website | xnetcoin.org |
| Telegram | t.me/xnethq |
| Twitter / X | @xnethq |
| Documentation | docs.xnetcoin.org |
| Explorer | explorer.xnetcoin.org |

---

*© 2025 XNET Network. All rights reserved.*  
*This document is for informational purposes only and does not constitute financial or investment advice.*