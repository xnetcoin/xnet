# Building and Running the XNET Node

## Prerequisites

Install the Rust toolchain and WASM compilation target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown
```

The Rust toolchain version is pinned in `rust-toolchain.toml` at the root of the
repository. `rustup` will automatically install the correct version.

---

## Building the Node

The `xnet-node` binary does not ship pre-compiled. It must be built from source.

```bash
# Clone
git clone https://github.com/xnetcoin/xnet
cd xnet

# Build only the node binary (recommended — avoids compiling unused crates)
cargo build --release -p xnet-node
```

Build time is approximately 10–20 minutes on a modern laptop (first build only;
incremental rebuilds are much faster).

After the build completes the binary is at:

```
./target/release/xnet-node
```

Verify it built correctly:

```bash
./target/release/xnet-node --version
```

---

## Running a Dev Node

A development node starts with a pre-funded test account (Alice) and produces blocks
even with a single validator. It does not require external peers or a chain spec file.

```bash
./target/release/xnet-node --dev
```

The node is available at:

| Protocol | Endpoint |
|---|---|
| WebSocket | `ws://127.0.0.1:9944` |
| HTTP RPC | `http://127.0.0.1:9944` |
| Ethereum RPC | `http://127.0.0.1:9944` |

Connect via [Polkadot.js Apps](https://polkadot.js.org/apps) → Settings → Custom endpoint
→ `ws://127.0.0.1:9944`.

Connect MetaMask with: RPC URL `http://127.0.0.1:9944`, Chain ID `2009`.

### With persistent state

By default `--dev` wipes the database on restart. Use `--base-path` to keep state:

```bash
./target/release/xnet-node --dev --base-path /tmp/xnet-dev
```

---

## Local Testnet (3 validators)

```bash
# Terminal 1 — Alice
./target/release/xnet-node \
  --chain local --alice \
  --port 30333 --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --validator

# Terminal 2 — Bob
./target/release/xnet-node \
  --chain local --bob \
  --port 30334 --rpc-port 9945 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<ALICE_PEER_ID> \
  --validator

# Terminal 3 — Charlie
./target/release/xnet-node \
  --chain local --charlie \
  --port 30335 --rpc-port 9946 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<ALICE_PEER_ID> \
  --validator
```

Replace `<ALICE_PEER_ID>` with the peer ID printed to stdout by Alice's terminal
on startup (looks like `12D3Koo...`).

---

## Running Tests

```bash
# All pallet unit tests
cargo nextest run

# Specific pallet
cargo nextest run -p pallet-block-reward
cargo nextest run -p pallet-zk-verifier
```

---

## Exporting a Chain Spec

```bash
# Human-readable spec
./target/release/xnet-node build-spec --chain dev > dev_spec.json

# Raw (binary-encoded) spec — required for actual deployment
./target/release/xnet-node build-spec --chain dev --raw > dev_spec_raw.json
```

---

## Benchmarking

```bash
cargo build --release --features runtime-benchmarks -p xnet-node

./target/release/xnet-node benchmark pallet \
  --chain dev \
  --pallet pallet_block_reward \
  --extrinsic "*" \
  --steps 50 \
  --repeat 20 \
  --output pallets/block-rewards/src/weights.rs
```
