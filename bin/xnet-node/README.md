# 💎 Xnetcoin (XNC) Node

> **The Obsidian Standard of Blockchain Technology.**
> A high-performance, secure, and scalable Layer-1 blockchain built with the Polkadot SDK.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Language](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-GPL3.0-blue)

## 📋 Overview

**Xnetcoin** is a next-generation blockchain protocol designed to offer lightning-fast transaction finality and robust security. Built on top of the battle-tested **Substrate** framework, Xnetcoin utilizes a hybrid consensus model (BABE + GRANDPA) and features a custom block reward mechanism.

Our mission is to provide a solid infrastructure for decentralized finance (DeFi) and secure digital assets, represented by our native token **XNC**.

## ✨ Key Features

* **🛡️ Hybrid Consensus:** Combines **BABE** for block production and **GRANDPA** for deterministic finality.
* **💰 Custom Block Rewards:** Features a specialized `block-rewards` pallet designed for sustainable tokenomics.
* **⚡ High Performance:** Optimized for high throughput using the latest Polkadot SDK.
* **🔄 Forkless Upgrades:** The runtime can be upgraded on-chain without hard forks.
* **❄️ Nix Supported:** Fully reproducible development environment using Nix flakes.

## 📂 Project Structure

Based on the Xnetcoin source tree:

* **`node/`**: The blockchain node logic (CLI, Service, RPC, Chain Spec). This is the "brain" of the node.
* **`runtime/`**: The blockchain state transition logic. This is the "heart" of the chain.
* **`pallets/`**: Custom modules specific to Xnetcoin.
    * `block-rewards`: Handles the logic for distributing XNC validation rewards.
* **`docs/`**: Documentation and setup guides (e.g., `rust-setup.md`).
* **`scripts/`**: Utility scripts for initialization and maintenance.

## 🚀 Getting Started

### Prerequisites

Please refer to [docs/rust-setup.md](docs/rust-setup.md) for detailed environment setup instructions. Generally, you need Rust and the Wasm toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### 🛠️ Build

Clone the repository and build the binary in release mode:
 ```bash
 git clone [https://github.com/xnetcoin/xnet.git]
cd xnet
```
Build the node:
```bash
cargo build --release
```

*Note: This may take a while depending on your hardware.*

### ⚡ Run in Development Mode

To start a single-node development chain (state is cleared on exit):

```bash
./target/release/xnet-node --dev
```

### 🌐 Run a Local Testnet

To run a local testnet with two nodes (Alice and Bob):

**Node 1 (Alice):**
```bash
./target/release/xnet-node \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator
```

**Node 2 (Bob):**
```bash
./target/release/xnet-node \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## 🛠 Interaction

You can interact with your node using the **Polkadot.js Apps** portal.

1.  Open [https://polkadot.js.org/apps/](https://polkadot.js.org/apps/)
2.  Click on the top left logo to switch networks.
3.  Choose **Development** -> **Local Node** (`127.0.0.1:9944`).
4.  Click **Switch**.

## 🪙 Tokenomics

* **Token Symbol:** XNC
* **Decimals:** 18
* **Consensus:** NPoS (Nominated Proof-of-Stake)

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1.  Fork the project.
2.  Create your feature branch (`git checkout -b feature/AmazingFeature`).
3.  Commit your changes (`git commit -m 'Add some AmazingFeature'`).
4.  Push to the branch (`git push origin feature/AmazingFeature`).
5.  Open a Pull Request.

## 📄 License

This project is licensed under the **GPL-3.0** License. See the [LICENSE](LICENSE) file for details.