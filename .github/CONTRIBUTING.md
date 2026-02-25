# Contributing to XNet

First off, thank you for considering contributing to XNet! It's people like you that make XNet such a powerful, secure, and decentralized network.

> ⚠️ **IMPORTANT NOTE:** XNet is currently in **Active Testnet Phase**. The core node, consensus, and runtime features are under heavy development and are not yet 100% complete. We are actively working on integrating Wasm/ink! smart contracts, Zero-Knowledge Proofs (ZKP), and Cross-Consensus Messaging (XCM). Please expect breaking changes and frequent updates.

XNet is a next-generation blockchain built on [Substrate](https://substrate.io/) and designed for high-performance decentralized applications (dApps). Our architecture leverages a Nominated Proof-of-Stake (NPoS) consensus mechanism and features full EVM and Wasm (ink!) compatibility, bridging the gap between Rust-native performance and the Ethereum developer ecosystem.

This document serves as the canonical source of truth for how to contribute to the XNet core protocol, runtime, and surrounding tooling.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Pull Requests (The Core Workflow)](#pull-requests-the-core-workflow)
- [EVM Grant Program](#evm-grant-program)
  - [Stage 1: Proposal Submission](#stage-1-proposal-submission)
  - [Stage 2: Core Team Review & Initial Approval](#stage-2-core-team-review--initial-approval)
  - [Stage 3: Milestone Delivery & Finalization](#stage-3-milestone-delivery--finalization)
- [Development Setup](#development-setup)
- [Coding Guidelines](#coding-guidelines)

---

## Code of Conduct

This project and everyone participating in it is governed by the [XNet Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the core team.

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please [follow the Bug Report template](ISSUE_TEMPLATE/bug_report.yml) and include as many details as possible:

*   **Use a clear and descriptive title** for the issue to identify the problem.
*   **Describe the exact steps which reproduce the problem** in as many details as possible.
*   **Provide specific examples** to demonstrate the steps.
*   **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
*   **Explain which behavior you expected to see instead and why.**
*   **Include your environment details** (OS, Rust compiler version, XNet node version).

> **Security Vulnerabilities:** If you find a security vulnerability, do NOT open an issue. Please read our [SECURITY.md](SECURITY.md) and report it responsibly.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When you are creating an enhancement suggestion, please [follow the Feature Request template](ISSUE_TEMPLATE/feature_request.yml) providing the following information:

*   **Use a clear and descriptive title** for the issue to identify the suggestion.
*   **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
*   **Explain why this enhancement would be useful** to most XNet users.

### Pull Requests (The Core Workflow)

We welcome PRs for bug fixes, features, and documentation improvements. Please adhere to the following workflow:

1.  **Fork** the repository and clone it locally.
2.  **Branch** from `main` (e.g., `git checkout -b fix/issue-number-description`).
3.  **Implement** your changes.
4.  **Test** your changes comprehensively. If modifying the runtime, ensure storage migrations and benchmarks are updated and passing.
5.  **Commit** using conventional commit messages (e.g., `fix(runtime): resolve state transition bug in staking pallet`).
6.  **Push** to your fork.
7.  **Submit a Pull Request** using our [Pull Request Template](pull_request_template.md).

All PRs require review from a core maintainer before being merged. Continuous Integration (CI) must pass cleanly.

---

## EVM Grant Program

XNet strongly supports the growth of its EVM ecosystem. If you are building a valuable product on XNet's EVM layer—be it DeFi protocols, NFT marketplaces, gaming infrastructure, or vital tooling—you may be eligible for a grant.

Our grant process is designed to be transparent, rigorous, and milestone-driven, operating in **3 distinct stages**:

### Stage 1: Proposal Submission & Pre-screening
To begin, you must submit a formal pitch outlining your project.
1.  Open a new issue using the **[Grant Proposal Template](ISSUE_TEMPLATE/grant_proposal.yml)**.
2.  Detail your project's architecture, target audience, and how it benefits the XNet ecosystem.
3.  Provide a clear breakdown of requested funding tied to specific, measurable milestones.

*The community and core team will review the proposal for feasibility and alignment with XNet's strategic goals.*

### Stage 2: Core Team Review & Initial Approval
If Stage 1 is successful, your proposal moves to deep technical and economic review by the XNet Grants Committee.
1.  The team may request architectural diagrams, team background verification, or code samples.
2.  If the project meets all stringent requirements, it receives **Initial Approval**.
3.  An initial tranche of funding (if stipulated in the milestone breakdown) may be released to bootstrap development.

### Stage 3: Milestone Delivery & Finalization
This is the execution phase.
1.  Your team develops the product according to the approved timeline.
2.  Code must be open-source (typically) or available for comprehensive audit by the XNet security team.
3.  Upon successful delivery and verification of each milestone, subsequent grant tranches are released.
4.  Once the final milestone is delivered and the product is live on XNet Mainnet (or successful Testnet deployment as agreed), the grant is finalized.

---

## Development Setup

As a Substrate-based chain, XNet requires a specific Rust toolchain environment.

### Prerequisites

You need the standard Rust toolchain and the WebAssembly (Wasm) compilation target. We strongly recommend using `rustup`.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required target and components
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

You will also need standard build tools (`cmake`, `clang`, `pkg-config`, `libssl-dev` on Linux).

### Building

To build the XNet node:

```bash
cargo build --release
```

To run the full test suite:

```bash
cargo test --all
```

## Coding Guidelines

We enforce strict coding standards to maintain the security and performance of the network.

*   **Rustfmt:** All code must be formatted using `rustfmt`. Run `cargo fmt --all` before committing.
*   **Clippy:** We treat warnings as errors in CI. Run `cargo clippy --all-targets --all-features -- -D warnings` locally.
*   **Documentation:** All public types, functions, and modules must be documented using rustdoc.
*   **Substrate Best Practices:** Always follow best practices for Substrate runtime development, particularly regarding *Weights*, *Storage Bounds*, and *Safe Math*.

## Stay Connected

*   **Website:** [xnetcoin.org](https://xnetcoin.org)
*   **Telegram:** [t.me/xnethq](https://t.me/xnethq)
*   **Twitter (X):** [@xnethq](https://x.com/xnethq)
*   **GitHub:** [xnetcoin](https://github.com/xnetcoin)

Thank you for contributing to the future of XNet!
