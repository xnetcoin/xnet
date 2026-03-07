// XNET Protocol
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

//! # XNET Node — Crate Overview
//!
//! This crate is the top-level documentation entry point for the XNET blockchain node.
//! It does not contain executable logic of its own; instead it describes the overall
//! architecture and guides contributors to the relevant sub-crates.
//!
//! ## Architecture
//!
//! The node is split into two major components — the **client** and the **runtime**:
//!
//! - The **client** handles networking, database, consensus coordination, RPC, and all
//!   host-side logic. Client crates are prefixed with `sc-` (substrate-client).
//!
//! - The **runtime** contains the application-specific state transition logic compiled
//!   to WebAssembly. Runtime crates are prefixed with `sp-` (primitives) or implemented
//!   as FRAME pallets under `pallet-` / `frame-`.
//!
//! The client and runtime communicate through two mechanisms:
//!
//! 1. **Runtime APIs** — the client calls into the Wasm runtime through typed API
//!    traits defined with the `impl_runtime_apis!` macro. The most fundamental of
//!    these is `Core`, which every compliant runtime must implement.
//!
//! 2. **Host functions** — the Wasm runtime calls back into the client for I/O
//!    operations such as storage reads/writes and cryptographic primitives, all
//!    defined under `sp-io`.
//!
//! This separation guarantees that the client is generic and reusable, while the
//! runtime encapsulates all chain-specific logic and can be upgraded on-chain
//! without a hard fork.
//!
//! ## Consensus
//!
//! XNET uses a hybrid consensus model:
//!
//! - **BABE** (Blind Assignment for Blockchain Extension) for block production.
//!   Validators are assigned slots through a VRF lottery, making authorship
//!   unpredictable to adversaries until the slot begins.
//!
//! - **GRANDPA** (GHOST-based Recursive ANcestor Deriving Prefix Agreement) for
//!   finality. Validators vote on chains — not individual blocks — and a chain is
//!   finalized once 2/3+ of the active validator set has committed to it.
//!
//! This combination provides strong liveness guarantees (blocks keep being produced
//! even if finality stalls) alongside deterministic safety (finalized blocks are
//! irreversible).
//!
//! ## Staking and Validator Selection
//!
//! XNET implements Nominated Proof-of-Stake (NPoS) using the Sequential Phragmén
//! election algorithm. Token holders nominate validators they trust; the algorithm
//! distributes stake across the elected set as evenly as possible to maximize
//! decentralization. Rewards and slashing both apply proportionally.
//!
//! Key on-chain parameters:
//!
//! - Maximum active validators: 100
//! - Era duration: 24 hours (1,800 blocks at 6s block time)
//! - Unbonding period: 28 eras (~28 days)
//! - Validator minimum bond: 8,000 XNET
//! - Nominator minimum bond: 1,000 XNET
//!
//! ## Smart Contracts
//!
//! XNET supports two independent smart contract environments:
//!
//! - **EVM** via the Frontier pallet stack. Any Solidity contract targeting the
//!   Ethereum Virtual Machine deploys without modification. XNET's EVM chain ID
//!   is `2009`. The full set of Ethereum precompiles (0x01–0x09) is available,
//!   including BN128 pairing for on-chain ZK proof verification.
//!
//! - **ink!** (WebAssembly) via `pallet-contracts`. Contracts are written in Rust
//!   using the `ink!` eDSL and compiled to Wasm. They benefit from Rust's type
//!   system and memory safety, and share the same storage deposit model.
//!
//! Both environments are first-class; neither is a secondary integration layer.
//!
//! ## Zero-Knowledge Proof System
//!
//! XNET implements ZK proof capabilities at two distinct layers:
//!
//! - **ZK Layer 1 (EVM)** — Solidity-based ZK verifier contracts deployable today.
//!   Compatible with the full Ethereum ZK toolchain: Circom, snarkjs, ZoKrates, Noir.
//!   BN128 precompiles (0x06–0x08) enable efficient on-chain verification.
//!
//! - **ZK Layer 2 (Native)** — `pallet-zk-verifier` implements Groth16 ZK-SNARKs
//!   directly in the Substrate runtime using BN254 elliptic curve arithmetic, written
//!   entirely in Rust without external cryptographic library dependencies.
//!   Features: nullifier registry (replay protection), block-level proof limits
//!   (DoS protection), full Circom/snarkjs compatibility.
//!
//! ## Fee Distribution
//!
//! Transaction fees are not paid to validators in full. XNET splits fees at the
//! pallet level using a custom `DealWithFees` type:
//!
//! - **60%** → on-chain Treasury
//! - **15%** → developer grant pool (a sub-account of Treasury)
//! - **25%** → block author (validator)
//!
//! This model funds protocol development and ecosystem grants sustainably from
//! ordinary transaction activity, without requiring external fundraising.
//!
//! ## Token Economics
//!
//! The native token is `XNET` with 18 decimal places. The emission schedule mirrors
//! Bitcoin's design:
//!
//! - Hard supply cap: **53,000,000 XNET**
//! - Initial block reward: **1,117 XNET** per block
//! - Halving interval: every **21,038,400 blocks** (~4 years at 6s block time)
//! - EVM Chain ID: **2009**
//! - SS58 Prefix: **888**
//!
//! The genesis premine is **6,000,000 XNET**, entirely locked via `pallet-vesting`:
//!
//! - 1,000,000 XNET — Founder (2.5 years linear vesting)
//! - 1,000,000 XNET — Investor Reserve (2.5 years linear vesting)
//! - 2,000,000 XNET — Ecosystem Fund (3 years linear vesting)
//! - 2,000,000 XNET — Treasury Reserve (governance-controlled)
//!
//! No premine token can be transferred until the vesting schedule allows it.
//!
//! ## Runtime Upgrades
//!
//! The XNET runtime is stored on-chain as a Wasm blob. Protocol upgrades are
//! enacted by replacing this blob through a governance-approved extrinsic. All
//! connected nodes automatically switch to the new runtime at the upgrade block,
//! with zero coordination and no network split.
//!
//! Currently the runtime is administered via a sudo key for rapid iteration during
//! the testnet phase. The sudo key will be removed before mainnet launch, at which
//! point all upgrades require on-chain governance approval.
//!
//! ## Wasm Build
//!
//! Runtime crates must compile to both native (for off-chain tools and tests) and
//! Wasm (for the on-chain blob). Substrate's convention of
//! `#![cfg_attr(not(feature = "std"), no_std)]` gates standard-library usage
//! behind a feature flag.
//!
//! The `substrate-wasm-builder` crate in `runtime/build.rs` invokes the Wasm
//! compiler automatically during `cargo build`. The resulting blob is placed at:
//!
//! ```text
//! target/{debug|release}/wbuild/xnet-runtime/xnet_runtime.wasm
//! ```
//!
//! ## Repository Layout
//!
//! ```text
//! xnet/
//! ├── .cargo/                       — Cargo config (build flags, target overrides)
//! ├── .config/                      — Project-level config
//! ├── .github/                      — CI workflows (lint, test, release, security audit)
//! ├── .maintain/                    — Maintenance scripts
//! ├── bin/                          — Node binary entrypoint
//! ├── contracts/                    — Example smart contracts (EVM + ink!)
//! ├── docker/                       — Docker & docker-compose configs
//! ├── docs/                         — Documentation (setup, contributing, security)
//! ├── ignition/                     — Deployment ignition scripts
//! ├── node/                         — Node source (CLI, service, RPC, chain_spec)
//! ├── pallets/                      — Custom XNET-specific FRAME pallets
//! │   ├── block-rewards/            — Bitcoin-style halving block rewards
//! │   └── zk-verifier/             — Native Groth16 ZK-SNARK verification
//! ├── primitives/                   — Shared types and traits
//! ├── runtime/                      — WASM runtime
//! │   └── src/
//! │       ├── lib.rs                — construct_runtime! macro, all pallet configs
//! │       └── precompiles.rs        — EVM precompile set (0x01–0x09)
//! ├── scripts/                      — Utility and deployment scripts
//! ├── src/
//! │   └── lib.rs                    — this file (documentation only)
//! ├── test/                         — Integration tests
//! ├── test-utils/                   — Shared test utilities
//! ├── utils/                        — General utilities
//! ├── xnet_wasm/                    — Wasm build artifacts
//! ├── xnet-privacy-contracts/       — Privacy contract examples
//! ├── zombienet/                    — Multi-node test network configs
//! ├── .gitignore
//! ├── .gitattributes
//! ├── .gitlab-ci.yml
//! ├── .markdownlintrc.json
//! ├── Cargo.toml                    — Workspace manifest
//! ├── Cargo.lock
//! ├── HEADER-APACHE2
//! ├── HEADER-GPL3
//! ├── LICENSE-APACHE2
//! ├── LICENSE-GPL3
//! └── package.json
//! ```
//!
//! ## Getting Started
//!
//! For runtime development, start with `frame_support` and the `construct_runtime!`
//! macro in `runtime/src/lib.rs`. For client-side customization (custom RPC, new
//! consensus adapter), look at `node/src/service.rs`.
//!
//! For environment setup, see [`docs/environment-setup.md`].
//!
//! [`docs/environment-setup.md`]: ../docs/environment-setup.md

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

/// Architectural overview of the XNET node — client, runtime, and their interface.
///
/// ## Client ↔ Runtime Communication
///
/// ```mermaid
/// graph TB
/// subgraph XNET Node
///     direction LR
///     subgraph Client
///         Networking
///         Database
///         Consensus["Consensus (BABE + GRANDPA)"]
///         RPC
///     end
///     subgraph Runtime["Runtime (Wasm)"]
///         subgraph FRAME
///             direction LR
///             Balances["pallet-balances"]
///             Staking["pallet-staking"]
///             EVM["pallet-evm (Frontier)"]
///             Contracts["pallet-contracts (ink!)"]
///             ZKVerifier["pallet-zk-verifier (Groth16)"]
///             BlockRewards["pallet-block-rewards"]
///             Treasury["pallet-treasury"]
///             Governance["pallet-democracy"]
///         end
///     end
///     Client --"runtime API"--> Runtime
///     Runtime --"host functions"--> Client
/// end
/// ```
///
/// The client is chain-agnostic. The runtime is the chain. When the runtime is
/// upgraded on-chain, the client picks up the new Wasm blob automatically at
/// the block where the upgrade was enacted — no binary distribution required.
pub mod xnet_diagram {}