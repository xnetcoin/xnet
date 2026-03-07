# XNET Protocol — Technical Whitepaper

**Version 1.0 — 2025**

Independent Layer 1 Blockchain | Dual-VM (EVM + WASM) | Dual ZK-Proof System | Hard Cap 53M

| | |
|---|---|
| Ticker | XNC |
| Website | https://xnetcoin.org |
| GitHub | https://github.com/xnetcoin/xnet |
| Telegram | https://t.me/xnethq |
| Twitter / X | https://x.com/xnethq |
| Email | info@xnetcoin.org |

---

## Disclaimer

This whitepaper is provided for informational and technical documentation purposes only. It does not constitute financial, investment, or legal advice of any kind. Nothing in this document represents an offer or solicitation to purchase, sell, or subscribe to any financial instrument or security.

XNC tokens are utility tokens used to participate in network consensus (staking), pay transaction fees, and access protocol services. They are not investment contracts and confer no ownership rights, dividends, or profit-sharing entitlements.

The XNET protocol is in active development. Technical specifications, tokenomics parameters, and roadmap milestones described herein reflect the current state of development and are subject to change. Readers are encouraged to verify all information against the canonical source code at https://github.com/xnetcoin/xnet.

Participation in any blockchain network involves risk, including but not limited to software vulnerabilities, regulatory changes, and market volatility. Readers should conduct independent due diligence before making any decisions.

---

## Abstract

XNET is an independent Layer 1 blockchain built from scratch on the Substrate framework. It is not a fork, not a template instantiation, and not a parachain — it operates with its own independent validator set, its own consensus, and its own security model.

The protocol integrates four capabilities that are typically found only in separate specialized chains: full EVM compatibility for Solidity developers, native WASM smart contracts via ink!, a dual zero-knowledge proof system, and Bitcoin-inspired deflationary tokenomics with a hard-capped supply of 53,000,000 XNC.

The dual ZK system is a key technical differentiator. The first ZK layer operates within the EVM environment — Solidity-based ZK contracts deployable today. The second layer is a native Substrate pallet (`pallet-zk-verifier`) implementing Groth16 ZK-SNARKs directly in the runtime, without external libraries or off-chain infrastructure. No equivalent open-source Groth16 pallet exists in the Substrate ecosystem.

XNET was developed by a single engineer over approximately twelve months. All source code is publicly available. The protocol is currently finalizing `pallet-zk-verifier` integration before public testnet launch.

---

## Table of Contents

1. Introduction
2. Problem Statement
3. Architecture Overview
4. Consensus: BABE + GRANDPA
5. Dual-VM Execution Environment
   - 5.1 EVM via Frontier
   - 5.2 WASM via ink!
6. Dual Zero-Knowledge Proof System
   - 6.1 ZK Layer 1 — EVM-Based ZK Contracts
   - 6.2 ZK Layer 2 — Native pallet-zk-verifier (Groth16)
   - 6.3 Comparison and Use Cases
7. Governance and Sudo Transition
8. XCM Cross-Chain Messaging
9. Tokenomics
   - 9.1 Supply and Halving Schedule
   - 9.2 Token Allocation
   - 9.3 Staking Parameters
   - 9.4 Fee Distribution
10. Developer Grants Program
11. Roadmap
12. Security Considerations
13. Conclusion
14. References and Contact

---

## 1. Introduction

XNET is an independent Layer 1 blockchain. It does not rely on Polkadot's relay chain for security, does not lease a parachain slot, and does not depend on any external validator set. XNET's security comes entirely from its own NPoS validator network.

While XNET uses Substrate as its runtime framework — the same technology used to build Polkadot, Kusama, and many production chains — the choice of Substrate is an engineering decision, not an architectural dependency. Substrate provides a proven, modular foundation for building custom blockchains. XNET takes full advantage of this modularity while remaining fully sovereign.

The protocol was designed around a single principle: give developers a single chain where they can build anything, without being forced to choose between execution environments, bridge assets across chains, or rely on off-chain infrastructure for cryptographic operations.

After approximately twelve months of development, the runtime is complete and all core features are operational in local testing. The immediate goal is completing `pallet-zk-verifier` integration, followed by public testnet launch.

---

## 2. Problem Statement

The current blockchain infrastructure landscape forces developers into false choices:

- EVM chains offer broad tooling compatibility but lock developers into Solidity's limitations and Ethereum's fee economics.
- WASM chains offer performance and safety guarantees but require abandoning the Ethereum developer ecosystem entirely.
- Privacy and ZK applications require either separate rollup infrastructure, trusted off-chain verifiers, or expensive custom cryptographic precompiles.
- Deflationary monetary policy is rare in smart contract platforms — most use inflationary token models that dilute holders indefinitely.

These are not unsolvable problems. They are the result of chains being built to serve narrow use cases. XNET's architecture treats all four capabilities as first-class citizens of the same runtime.

---

## 3. Architecture Overview

| Component | Details |
|---|---|
| Framework | Substrate (Rust) — written from scratch, not forked |
| Consensus | BABE (block production) + GRANDPA (finality) |
| Staking | Nominated Proof-of-Stake (NPoS) |
| VM Layer 1 | EVM via Frontier — full Ethereum compatibility |
| VM Layer 2 | WASM via ink! — Rust smart contracts |
| ZK Layer 1 | EVM-based ZK contracts (Solidity, live today) |
| ZK Layer 2 | pallet-zk-verifier — native Groth16 (in integration) |
| Governance | Sudo (testnet) → on-chain governance before mainnet |
| Interoperability | XCM — testnet phase |
| Chain Type | Independent Layer 1 — sovereign validator set |
| Supply | 53,000,000 XNC hard cap — immutable, runtime-encoded |
| EVM Chain ID | 2009 |
| SS58 Prefix | 888 |

---

## 4. Consensus: BABE + GRANDPA

XNET uses a hybrid consensus model combining two complementary protocols.

### BABE — Block Production

BABE (Blind Assignment for Blockchain Extension) is a slot-based block production protocol. Validators are assigned block authoring slots using Verifiable Random Functions (VRF), ensuring that no validator can predict future slot assignments in advance. This prevents targeted denial-of-service attacks against upcoming block producers.

Each epoch, the VRF lottery assigns primary and secondary slots. Primary slots go to validators whose VRF output falls below a threshold; secondary slots provide fallback liveness guarantees. BABE provides probabilistic safety — the probability of a block being reversed decreases exponentially with the number of subsequent blocks.

### GRANDPA — Finality

GRANDPA (GHOST-based Recursive Ancestor Deriving Prefix Agreement) operates as a finality gadget layered above BABE. Rather than finalizing individual blocks, GRANDPA finalizes entire chains — allowing it to batch-finalize dozens of blocks in a single round under normal network conditions.

GRANDPA achieves Byzantine fault tolerance with a 2/3 honest-validator threshold. If more than 2/3 of validators are online and honest, the protocol guarantees that finalized blocks will never be reversed. This makes XNET suitable for high-value applications requiring transaction irreversibility.

| Parameter | Value |
|---|---|
| Block time | 6 seconds |
| Finality | ~12 seconds |
| Block size | 5 MB |
| Gas limit | 75,000,000 / block |
| Era duration | 24 hours (1,800 blocks) |
| Unbonding period | 28 eras (~28 days) |
| Max active validators | 100 |

---

## 5. Dual-VM Execution Environment

XNET runs two fully independent smart contract execution environments on the same chain state. Both VMs share the same account system, the same native token (XNC), and the same on-chain storage layer.

### 5.1 EVM via Frontier

The Frontier pallet provides a complete Ethereum-compatible execution environment. Every Solidity contract that runs on Ethereum, Polygon, or any EVM-compatible chain deploys to XNET without modification.

- Full EVM opcode compatibility — identical execution semantics to Ethereum
- Standard JSON-RPC interface — eth_, net_, web3_ namespaces
- MetaMask, WalletConnect, and all standard Web3 wallets connect natively
- Hardhat, Truffle, Remix, Foundry — all Ethereum dev tools work without changes
- ERC-20, ERC-721, ERC-1155, and all standard token interfaces supported
- Full set of Ethereum precompiles (0x01–0x09) including BN128 pairing

EVM gas is denominated in XNC. Existing Solidity code requires no adaptation.

### 5.2 WASM via ink!

The `pallet-contracts` module enables smart contracts written in Rust, compiled to WebAssembly. ink! is the primary contract framework, providing an ergonomic Rust DSL with full access to the Substrate storage API.

- Rust-based development with strong type safety and compile-time guarantees
- Deterministic execution with precise gas metering
- Sandbox isolation — contracts cannot access host system resources
- Cross-contract calls with typed interfaces
- On-chain upgradeable contracts via code hash indirection

---

## 6. Dual Zero-Knowledge Proof System

XNET implements zero-knowledge proof capabilities at two distinct layers of the protocol stack. This dual approach provides both immediate developer accessibility and long-term cryptographic depth.

### 6.1 ZK Layer 1 — EVM-Based ZK Contracts

The first ZK layer operates within XNET's EVM environment. Solidity-based ZK verification contracts — including implementations of Groth16, PLONK, and other common proof systems — deploy and execute on XNET exactly as they would on Ethereum.

This layer is fully operational today. Developers familiar with Ethereum-based ZK tooling (snarkjs, circom, ZoKrates, Noir) can deploy their existing contracts to XNET without modification.

- Immediate availability — no waiting for pallet integration
- Full Ethereum ZK toolchain compatibility
- Solidity verifier contracts for Groth16, PLONK, FFlonk
- BN128 precompiles (0x06–0x08) for efficient on-chain verification

### 6.2 ZK Layer 2 — Native pallet-zk-verifier (Groth16)

The second ZK layer is `pallet-zk-verifier` — a native Substrate pallet implementing Groth16 ZK-SNARK verification directly in the blockchain runtime. This pallet is currently in final integration into the XNET runtime.

**What makes this different from EVM-based ZK:**

EVM-based ZK verification executes inside the sandboxed EVM environment, consuming EVM gas and subject to EVM execution constraints. Native runtime ZK verification executes at the consensus layer — outside the VM sandbox, with direct access to the runtime state machine.

- Lower verification overhead — runtime execution bypasses EVM interpreter overhead
- Direct storage access — verified proofs can update chain state atomically
- No Solidity dependency — pure Rust implementation
- Deeper protocol integration — proof verification tied directly to runtime state

`pallet-zk-verifier` implements the complete Groth16 protocol using pairing-based elliptic curve arithmetic over the BN254 curve, written entirely in Rust with no external cryptographic library dependencies.

- Full Groth16 verifier — supports arbitrary arithmetic circuits compiled with Circom
- Nullifier registry — prevents proof replay attacks
- Block-level proof limits — DoS protection
- SnarkJS compatible — standard Circom/SnarkJS proofs verify on-chain

No comparable open-source Groth16 pallet exists in the Substrate ecosystem. Upon completion, `pallet-zk-verifier` will be available for any Substrate-based chain to integrate.

### 6.3 Comparison and Use Cases

| | ZK Layer 1 (EVM) | ZK Layer 2 (Native) |
|---|---|---|
| Status | Live | In final integration |
| Language | Solidity | Rust |
| Execution | EVM sandbox | Runtime (native) |
| Proof system | Any EVM-compatible | Groth16 |
| Toolchain | snarkjs, circom, ZoKrates | Circom + SnarkJS |
| Best for | Solidity ZK apps | Deep protocol integration |

---

## 7. Governance and Sudo Transition

During testnet operation, the XNET runtime is administered via a sudo key — a standard practice for early-stage Substrate chains that allows rapid iteration and runtime upgrades without requiring a full governance referendum for every change.

The sudo key is a development tool, not a permanent feature. Its removal is a hard prerequisite for mainnet launch. No mainnet will launch with sudo active. All sudo actions are fully on-chain and auditable by anyone.

**Post-sudo governance:**

- Runtime upgrades — require supermajority referendum approval
- Treasury disbursements — community proposal and approval process
- Parameter adjustments — governance-gated with time locks
- Validator set management — handled by NPoS election, not governance

The 2,000,000 XNC treasury reserve allocated at genesis is controlled exclusively through governance from day one. No individual key can access these funds.

---

## 8. XCM Cross-Chain Messaging

Following `pallet-zk-verifier` integration and testnet launch, XNET will implement XCM — the Cross-Consensus Messaging format developed by Parity Technologies. XCM integration is planned during the testnet phase, before mainnet.

XCM enables XNET to send and receive assets and messages to any XCM-compatible chain without trusted bridge intermediaries. Because XNET is built on Substrate, it shares the same runtime architecture as all Polkadot/Kusama parachains, making XCM integration straightforward.

---

## 9. Tokenomics

XNET's monetary policy is modeled on Bitcoin's core principles: a fixed maximum supply, algorithmic emission reduction (halving), and no ability for any governance action to increase the supply cap. All supply parameters are hardcoded in the Rust runtime source code.

### 9.1 Supply and Halving Schedule

| Parameter | Value |
|---|---|
| Token Name / Ticker | XNET / XNC |
| Maximum Supply | 53,000,000 XNC — immutable, encoded in runtime |
| Genesis Premine | 6,000,000 XNC (11.32% of total supply) |
| Mining Supply | 47,000,000 XNC via block rewards over ~100 years |
| Block Reward (Years 1–4) | 1,117 XNC per block |
| Halving Interval | Every ~4 years (21,038,400 blocks) |
| Block Reward (Years 5–8) | ~558 XNC per block |
| Block Reward (Years 9–12) | ~279 XNC per block |
| Emission End | ~Year 100 — hard cap reached |
| Liquid at Genesis | 0 — all premine locked via pallet-vesting |

### 9.2 Token Allocation

| Allocation | Amount (XNC) | % Supply | Lock / Vesting |
|---|---|---|---|
| Founder | 1,000,000 | 1.89% | 2.5 years linear |
| Investor Reserve | 1,000,000 | 1.89% | 2.5 years linear |
| Ecosystem Fund | 2,000,000 | 3.77% | 3 years linear |
| — of which: Developer Grants | 1,000,000 | 1.89% | Governance-released |
| Treasury Reserve | 2,000,000 | 3.77% | Governance-controlled |
| Block Rewards | 47,000,000 | 88.68% | ~100 years via halving |
| **TOTAL** | **53,000,000** | **100%** | |

The ecosystem fund allocates **1,000,000 XNC specifically for developer grants** — disbursed through on-chain governance to projects building on XNET. All vesting is enforced at the protocol level via `pallet-vesting`. No key can unlock early.

### 9.3 Staking Parameters

| Parameter | Value |
|---|---|
| Staking Model | Nominated Proof-of-Stake (NPoS) |
| Validator Minimum Bond | 8,000 XNC |
| Nominator Minimum Bond | 1,000 XNC |
| Validator Election | Phragmen algorithm (proportional representation) |
| Slashing | Yes — equivocation and offline behavior penalized |
| Max Active Validators | 100 |
| Unbonding Period | 28 eras (~28 days) |

### 9.4 Fee Distribution

Transaction fees are automatically split at the protocol level on every block. This distribution is hardcoded and cannot be changed by any single key:

| Recipient | Share | Purpose |
|---|---|---|
| Treasury | 60% | Infrastructure, tooling, ecosystem development |
| Developer Grants | 15% | Direct allocation to grants program |
| Validators | 25% | Distributed proportionally to active validators |

---

## 10. Developer Grants Program

XNET allocates **1,000,000 XNC** from the ecosystem fund and **15% of all transaction fees** to support developers building on the protocol. Grants are disbursed through on-chain governance — transparent, community-validated, and fully on-chain.

**Eligible projects:**

- DeFi protocols — DEXs, lending markets, stablecoins, yield aggregators
- NFT infrastructure — minting platforms, marketplaces, provenance systems
- ZK applications — privacy tools, identity systems, provable computation using `pallet-zk-verifier`
- Developer tooling — SDKs, indexers, testing frameworks, debugging tools
- Bridges and interoperability — trustless asset bridges, cross-chain messaging
- User-facing applications — wallets, explorers, dashboards, analytics
- Infrastructure — RPC providers, archive nodes, monitoring systems

**Evaluation criteria:**

- Technical merit — is the implementation sound and auditable?
- Open source — all grant-funded code must be publicly licensed
- Ecosystem value — does it make XNET more useful for developers or users?
- Sustainability — does the project have a realistic path to self-sufficiency?

---

## 11. Roadmap

**Now — ZKP Integration**

- `pallet-zk-verifier` final integration and testing
- Groth16 benchmark suite
- Circom reference circuit

**Phase 1 — Testnet Launch**

- Public testnet — permissionless validator participation
- Block explorer and faucet
- Public RPC endpoint
- xnetcoin.org full launch
- Developer documentation
- Community channels active — Telegram, Discord, Twitter

**Phase 2 — XCM (during Testnet)**

- XCM cross-chain messaging integration
- Cross-chain asset transfers
- Interoperability with Substrate ecosystem

**Phase 3 — Mainnet**

- 60+ consecutive days of stable testnet
- Sudo key removal — full on-chain governance
- External security review
- Mainnet genesis
- Web wallet — Polkadot.js + MetaMask compatible
- JavaScript SDK (xnet.js)

**Phase 4 — Ecosystem Growth**

- Native DEX
- Ethereum bridge
- CEX listings
- Mobile wallet
- zkEVM exploration

---

## 12. Security Considerations

**Consensus safety:** GRANDPA's 2/3 threshold means up to one-third of validators can be offline or Byzantine without compromising finality. BABE's VRF-based slot assignment prevents any observer from predicting the next block producer.

**Validator incentive alignment:** Validators bond a minimum of 8,000 XNC as slashable collateral. Equivocation or sustained offline behavior results in proportional slashing, creating direct financial cost for attacking the network.

**Sudo key risk mitigation:** All sudo actions are publicly visible on-chain in real time. The sudo key cannot access vesting-locked funds or override fee distribution. Sudo removal before mainnet is a hard protocol commitment.

**ZK proof security:** `pallet-zk-verifier` relies on the hardness of the discrete logarithm problem on the BN254 elliptic curve — the same assumption underlying Ethereum's bn256 precompile. The nullifier registry prevents proof replay. Block-level verification limits prevent computational DoS.

**Smart contract isolation:** EVM and WASM contracts execute in isolated sandboxes with deterministic gas metering. Neither VM has direct access to validator keys or consensus state.

**Upgrade safety:** Post-sudo, all runtime upgrades require on-chain governance approval with an enforced time lock. Upgrades are applied via Substrate's forkless upgrade mechanism.

---

## 13. Conclusion

XNET is a fully independent Layer 1 blockchain that took approximately twelve months to build from scratch. It is not a fork, not a template, and not a parachain. It has its own validators, its own consensus, and its own security.

The protocol delivers what most chains treat as separate concerns — EVM compatibility, WASM contracts, dual ZK-proof infrastructure, and deflationary tokenomics — in a single unified runtime. The native Groth16 pallet, once complete, will be the only open-source implementation of its kind in the Substrate ecosystem.

The technical foundation is complete. `pallet-zk-verifier` is in final integration. After that comes testnet, XCM, and mainnet — in that order.

All code is public. All tokenomics are on-chain. All parameters are auditable.

---

## 14. References and Contact

| | |
|---|---|
| Source Code | https://github.com/xnetcoin/xnet |
| Website | https://xnetcoin.org |
| Telegram | https://t.me/xnethq |
| Twitter / X | https://x.com/xnethq |
| Email | info@xnetcoin.org |

**Technical references:**

- Substrate Documentation — https://docs.substrate.io
- Frontier EVM Pallet — https://github.com/paritytech/frontier
- ink! Smart Contracts — https://use.ink
- BABE Consensus — https://research.web3.foundation/Polkadot/protocols/block-production/Babe
- GRANDPA Finality — https://github.com/w3f/consensus/blob/master/pdf/grandpa.pdf
- Groth16 ZK-SNARKs — Groth, J. "On the Size of Pairing-Based Non-interactive Arguments." EUROCRYPT 2016.
- BN254 Elliptic Curve — EIP-196, EIP-197
- Circom Compiler — https://github.com/iden3/circom
- SnarkJS — https://github.com/iden3/snarkjs
- NPoS and Phragmen — https://research.web3.foundation/Polkadot/protocols/NPoS
- XCM Format — https://github.com/paritytech/xcm-format

---

*A note on these docs: English isn't my first language. The last twelve months went into the code, not into writing. I used AI to help put this whitepaper together — to make sure the technical details were presented clearly and nothing important was left out. The architecture, the consensus design, the ZK implementation, the tokenomics — every decision in here is mine. The words got some help. If you want to know what this project really is, read the commits.*