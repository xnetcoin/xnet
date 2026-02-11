# 💎 Xnetcoin (XNC)

> **The Obsidian Standard of Blockchain Technology.**
> High-performance, secure, and scalable Layer-1 blockchain built with the Polkadot SDK.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)
![License](https://img.shields.io/badge/license-GPL3.0-blue)

## 📋 Overview

**Xnetcoin** is a next-generation blockchain protocol designed to offer lightning-fast transaction finality and robust security. Built on top of the battle-tested **Substrate** framework, Xnetcoin combines the speed of **BABE** block production with the deterministic finality of **GRANDPA**.

Our mission is to provide a solid infrastructure for decentralized finance (DeFi) and secure digital assets, represented by our native token **XNC**.

## ✨ Key Features

* **🛡️ Hybrid Consensus:** Utilizes **BABE** (Blind Assignment for Blockchain Extension) for block production and **GRANDPA** (GHOST-based Recursive Ancestor Deriving Prefix Agreement) for finality.
* **⚡ High Performance:** Optimized for high throughput and low latency.
* **🔄 Forkless Upgrades:** The chain runtime can be upgraded without hard forks, ensuring network stability.
* **💎 Tokenomics:** Deflationary mechanics with a custom block reward halving schedule.
* **🔌 Interoperability:** Designed to be compatible with the broader Polkadot and Substrate ecosystem.

## 🪙 Tokenomics

* **Token Name:** Xnetcoin
* **Symbol:** XNC
* **Decimals:** 18
* **Total Supply Cap:** ~53,000,000 XNC
* **Premine:** 6,000,000 XNC (Reserved for Foundation & Development)
* **Validation:** Proof-of-Stake (NPoS)

## 🚀 Getting Started

Follow these steps to get your local Xnetcoin node up and running.

### Prerequisites

Ensure you have Rust and the support software installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
1. Build the Node
Clone the repository and build the binary (this may take a while):

Bash
git clone [https://github.com/your-username/xnetcoin.git](https://github.com/your-username/xnetcoin.git)
cd xnetcoin
cargo build --release
2. Run in Development Mode
To start a single-node development chain (state is cleared on exit):

Bash
./target/release/xnet-node --dev
3. Run a Local Testnet
To run a local testnet with two nodes (Alice and Bob):

Node 1 (Alice):

Bash
./target/release/xnet-node \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator
Node 2 (Bob):

Bash
./target/release/xnet-node \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
🛠 Interaction
You can interact with your node using the Polkadot.js Apps portal.

Open https://polkadot.js.org/apps/

Click on the top left logo to switch networks.

Choose Development -> Local Node (127.0.0.1:9944).

Click Switch.

📂 Project Structure
node/: The blockchain node logic (CLI, Service, RPC, Chain Spec).

runtime/: The blockchain logic (Pallets, Runtime configurations).

pallets/: Custom modules specific to Xnetcoin.

🤝 Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

Fork the project.

Create your feature branch (git checkout -b feature/AmazingFeature).

Commit your changes (git commit -m 'Add some AmazingFeature').

Push to the branch (git push origin feature/AmazingFeature).

Open a Pull Request.

📄 License
This project is licensed under the GPL-3.0 License.