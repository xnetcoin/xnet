# Contributing to XNET

Thank you for your interest in contributing to the XNET protocol. As an open-source decentralized network, we welcome contributions from the community to improve the core node, runtime pallets, and associated tooling.

XNET is a high-performance blockchain built using [Substrate](https://substrate.io/). This document serves as the canonical source of truth for how to contribute to the XNET core repository.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Pull Requests (The Core Workflow)](#pull-requests-the-core-workflow)
- [Development Setup](#development-setup)
- [Coding Guidelines](#coding-guidelines)

---

## Code of Conduct

This project and everyone participating in it is governed by the [XNET Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check the existing issues. When you are creating a bug report, please follow the Bug Report template and include:

*   **Exact steps to reproduce the problem** 
*   **Expected vs Actual behavior**
*   **Environment details** (OS, Rust compiler version, XNET node version, branch/commit).

> **Security Vulnerabilities:** If you find a security vulnerability, do NOT open a public issue. Please read our [SECURITY.md](SECURITY.md) and report it responsibly.

### Suggesting Enhancements

Enhancement suggestions and protocol changes are tracked as GitHub issues. When proposing an architectural change, please structure it as an XNET Improvement Proposal (XIP).

### Pull Requests (The Core Workflow)

We welcome PRs for bug fixes, features, and documentation improvements. Please adhere to the following workflow:

1.  **Fork** the repository and clone it locally.
2.  **Branch** from `main` (e.g., `git checkout -b fix/issue-number-description`).
3.  **Implement** your changes.
4.  **Test** your changes comprehensively. If modifying the runtime, ensure storage migrations and benchmarks are updated and passing.
5.  **Commit** using conventional commit messages (e.g., `fix(runtime): resolve state transition bug in staking pallet`).
6.  **Push** to your fork.
7.  **Submit a Pull Request** using our Pull Request Template.

All PRs require review from a core maintainer before being merged. Continuous Integration (CI) must pass cleanly.

---

## Development Setup

As a Substrate-based chain, XNET requires a specific Rust toolchain environment.

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

To build the XNET node:

```bash
# Build the node in release mode
cargo build --release
```

To run the full test suite:

```bash
# Run all workspace tests
cargo test --all
```

## Coding Guidelines

We enforce strict coding standards to maintain the security and performance of the network.

*   **Rustfmt:** All code must be formatted using `rustfmt`. Run `cargo fmt --all` before committing.
*   **Clippy:** We treat warnings as errors in CI. Run `cargo clippy --workspace --all-targets -- -D warnings` locally.
*   **Documentation:** All public types, functions, and modules must be documented using rustdoc.
*   **Substrate Best Practices:** Always follow best practices for Substrate runtime development, particularly regarding *Weights*, *Storage Bounds*, and *Safe Math*.
