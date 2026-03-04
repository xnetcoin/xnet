# Rust & Development Environment Setup

This guide installs everything needed to build and run the XNET node from source.

---

## Supported Operating Systems

| OS | Status |
|----|--------|
| Ubuntu 20.04 / 22.04 / 24.04 | ✅ Recommended |
| Debian 11 / 12 | ✅ Supported |
| macOS 12+ (Intel & Apple Silicon) | ✅ Supported |
| Windows 11 (WSL2) | ⚠️ Via WSL2 only |
| Windows native | ❌ Not supported |

---

## Step 1 — Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, choose option `1` (default installation).

Then reload your shell:

```bash
source "$HOME/.cargo/env"
```

Verify:

```bash
rustc --version
# rustc 1.75.0 (or newer)

cargo --version
# cargo 1.75.0 (or newer)
```

---

## Step 2 — Add WASM Target

Substrate compiles the runtime to WebAssembly. Add the WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

Verify:

```bash
rustup target list --installed | grep wasm
# wasm32-unknown-unknown
```

---

## Step 3 — Install System Dependencies

### Ubuntu / Debian

```bash
sudo apt update && sudo apt install -y \
  git \
  clang \
  curl \
  libssl-dev \
  llvm \
  libudev-dev \
  make \
  protobuf-compiler \
  pkg-config \
  build-essential
```

### macOS

Install [Homebrew](https://brew.sh) first if not already installed:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Then install dependencies:

```bash
brew install openssl protobuf llvm cmake
```

Add LLVM to your PATH:

```bash
echo 'export PATH="/opt/homebrew/opt/llvm/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Windows (WSL2)

1. Install WSL2: open PowerShell as Administrator and run:
   ```powershell
   wsl --install
   ```
2. Restart your computer
3. Open Ubuntu from the Start menu
4. Follow the **Ubuntu / Debian** steps above inside WSL2

---

## Step 4 — Verify Protobuf Compiler

Substrate's networking layer requires `protoc` version 3.15 or newer:

```bash
protoc --version
# libprotoc 3.21.x (or newer)
```

If the version is too old, install manually:

```bash
# Ubuntu — remove old version and install latest
sudo apt remove protobuf-compiler
sudo apt install -y protobuf-compiler
```

---

## Step 5 — Clone and Build XNET Node

```bash
# Clone the repository
git clone https://github.com/xnetcoin/xnet-node
cd xnet-node

# Standard release build
cargo build --release
```

First build takes 10–20 minutes as it compiles all dependencies.

```bash
# Build with benchmark support (needed for weight generation)
cargo build --release --features runtime-benchmarks
```

Compiled binary location:

```
./target/release/xnet-node
```

---

## Step 6 — Run a Development Node

```bash
./target/release/xnet-node --dev
```

You should see output like:

```
2025-01-01 00:00:00 XNET Protocol Node
2025-01-01 00:00:00 ✌️  version 1.0.0
2025-01-01 00:00:00 ❤️  by XNET Protocol Team
2025-01-01 00:00:00 📋 Chain specification: XNET Development
2025-01-01 00:00:00 🏷  Node name: Alice
2025-01-01 00:00:00 👤 Role: AUTHORITY
2025-01-01 00:00:00 💾 Database: RocksDb
2025-01-01 00:00:00 🔨 Initializing Genesis block/state
2025-01-01 00:00:00 🚀 Starting consensus session on top of parent
```

The node is producing blocks. Connect at `ws://127.0.0.1:9944`.

---

## Optional Tools

### cargo-contract — for ink!/WASM smart contracts

```bash
cargo install cargo-contract --force --locked
```

Verify:

```bash
cargo contract --version
# cargo-contract 4.x.x
```

### Hardhat — for Solidity/EVM contracts

```bash
# Requires Node.js 18+
node --version

npm install --save-dev hardhat
npx hardhat init
```

### subkey — for key generation and inspection

```bash
cargo install subkey --locked
```

Generate a new keypair:

```bash
subkey generate --scheme sr25519
```

---

## Rust Toolchain Management

XNET uses a stable Rust toolchain. The required version is pinned in `rust-toolchain.toml`:

```bash
# Check active toolchain
rustup show

# Update to latest stable
rustup update stable

# If a specific version is required by rust-toolchain.toml, rustup installs it automatically
cargo build --release
```

---

## Troubleshooting

### `error: linker 'cc' not found`

```bash
sudo apt install build-essential
```

### `error: failed to run custom build command for 'librocksdb-sys'`

```bash
sudo apt install clang libclang-dev
```

### `error: Could not find directory of OpenSSL installation`

```bash
# Ubuntu
sudo apt install libssl-dev pkg-config

# macOS
export OPENSSL_DIR=$(brew --prefix openssl)
```

### `error: protoc not found` or `protoc: command not found`

```bash
sudo apt install protobuf-compiler

# Verify version >= 3.15
protoc --version
```

### `WASM binary not available`

The WASM runtime blob was not compiled. Make sure you ran:

```bash
cargo build --release
```

If the error persists:

```bash
# Force recompile the runtime
touch runtime/src/lib.rs
cargo build --release
```

### Build takes too long / runs out of memory

Substrate builds are memory-intensive. If your machine has less than 8 GB RAM:

```bash
# Limit parallel compile jobs to reduce memory usage
CARGO_BUILD_JOBS=2 cargo build --release
```

---

## Hardware Requirements

| | Minimum | Recommended |
|--|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16+ GB |
| Disk (build) | 50 GB | 100 GB SSD |
| Disk (validator node) | 100 GB | 500 GB SSD |
| Network | 10 Mbps | 100+ Mbps |

---

## Next Steps

Once your environment is set up and the node is running:

- [README.md](../README.md) — full node documentation
- Connect [Polkadot.js Apps](https://polkadot.js.org/apps) to `ws://127.0.0.1:9944`
- Deploy your first ink! contract with `cargo contract`
- Deploy your first Solidity contract with Hardhat using Chain ID `2009`

---

*For questions and support: [t.me/xnethq](https://t.me/xnethq)*
