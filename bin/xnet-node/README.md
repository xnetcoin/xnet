# XNET Node

<div align="center">

[![Substrate](https://img.shields.io/badge/Substrate-Framework-green?style=flat-square)](https://substrate.io)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square)](https://rustup.rs)
[![EVM](https://img.shields.io/badge/EVM-Chain%20ID%202009-blue?style=flat-square)](https://xnetcoin.org)
[![ink!](https://img.shields.io/badge/ink!-WASM%20Contracts-purple?style=flat-square)](https://use.ink)
[![License](https://img.shields.io/badge/License-Apache%202.0-lightgrey?style=flat-square)](LICENSE)

**A high-performance Layer 1 blockchain node built on Substrate with dual smart contract support (EVM + ink!/WASM), native Zero-Knowledge Proof verification, and Bitcoin-style tokenomics.**

[xnetcoin.org](https://xnetcoin.org) · [t.me/xnethq](https://t.me/xnethq) · [@xnethq](https://x.com/xnethq)

</div>

---

## What is XNET?

XNET is a Layer 1 blockchain that solves the three core trade-offs of existing chains:

| Problem | Solution |
|---------|----------|
| Ethereum is compatible but has no privacy | XNET has EVM compatibility **and** native ZKP |
| Privacy chains lack developer tooling | XNET supports MetaMask, Hardhat, Foundry out of the box |
| Chains are isolated from each other | XNET uses XCM for trustless cross-chain messaging |

**Hard supply cap: 53,000,000 XNET. Bitcoin-style halving. No inflation beyond block rewards.**

---

## Repository Structure

```
xnet-node/
├── docs/                        # Extended documentation
│   ├── rust-setup.md            # Environment setup guide
│
├── node/                        # Node binary crate
│   └── src/
│       ├── main.rs              # Entry point
│       ├── cli.rs               # CLI argument definitions
│       ├── command.rs           # CLI dispatcher
│       ├── chain_spec.rs        # Genesis configs (dev / local / mainnet)
│       ├── service.rs           # Node service — networking, consensus, DB
│       ├── rpc.rs               # JSON-RPC (Substrate + Ethereum endpoints)
│       └── benchmarking.rs      # Benchmark helpers (feature-gated)
│
├── pallets/                     # Custom FRAME pallets
│   ├── block-rewards/           # Bitcoin-style block reward with halving
│   └── zk-verifier/             # Zero-Knowledge Proof verification pallet
│
└── runtime/                     # WASM runtime crate
    ├── src/
    │   ├── lib.rs               # All pallet configs wired together
    │   └── precompiles.rs       # EVM precompile set (0x01–0x09)
    └── build.rs                 # Compiles runtime to WASM blob
```

---

## Quick Start

> **First time?** Follow [docs/rust-setup.md](docs/rust-setup.md) to install all dependencies.

```bash
# 1. Clone
git clone https://github.com/xnetcoin/xnet
cd xnet

# 2. Build
cargo build --release

# 3. Run a local dev node
./target/release/xnet-node --dev
```

Node is running. Connect at `ws://127.0.0.1:9944`.

---

## Running Nodes

### Development — single node, no peers

```bash
./target/release/xnet-node --dev
```

- Alice is the sole validator
- State resets on every restart
- Best for smart contract development and testing

```bash
# Persist state between restarts
./target/release/xnet-node --dev --base-path /tmp/xnet-dev
```

### Local Testnet — 3 validators (Alice, Bob, Charlie)

Open three terminals:

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

### Mainnet / Public Testnet

```bash
./target/release/xnet-node \
  --chain mainnet \
  --name "my-node" \
  --port 30333 \
  --rpc-port 9944 \
  --validator
```

---

## Connect Wallets & Tools

### Polkadot.js Apps

1. Open [polkadot.js.org/apps](https://polkadot.js.org/apps)
2. Settings → Custom endpoint → `ws://127.0.0.1:9944`

### MetaMask (EVM)

Add a custom network:

| Field | Value |
|-------|-------|
| Network Name | XNET |
| RPC URL | `http://127.0.0.1:9944` |
| Chain ID | `2009` |
| Symbol | `XNET` |
| Explorer | `https://explorer.xnetcoin.org` |

---

## Smart Contracts

### Solidity (EVM) — deploy any Ethereum contract unchanged

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

### ink! (WASM) — Rust-based contracts

```bash
# Install tooling
cargo install cargo-contract --force

# Create, build, deploy
cargo contract new my_contract
cd my_contract
cargo contract build --release
cargo contract instantiate \
  --contract target/ink/my_contract.contract \
  --constructor new \
  --suri //Alice \
  --url ws://127.0.0.1:9944
```

---

## Custom Pallets

### `pallet-block-rewards`

 block reward emission with halving:

| Parameter | Value |
|-----------|-------|
| Initial reward | 1.117 XNET per block |
| Halving interval | 21,038,400 blocks (~4 years) |
| Hard cap | 53,000,000 XNET |
| Reward stops | When total issuance hits the hard cap |

### `pallet-zk-verifier`

Native on-chain Zero-Knowledge Proof verification:

- Verifies zk-SNARK proofs (Groth16) using the BN254 elliptic curve
- Enables privacy transactions and confidential smart contracts
- Complements the EVM BN128 precompiles (`0x06`–`0x08`)

---

## EVM Precompiles

| Address | Name | Used by |
|---------|------|---------|
| `0x01` | ECRecover | Wallets, signature verification |
| `0x02` | SHA-256 | Bitcoin bridge, hashing |
| `0x03` | RIPEMD-160 | Bitcoin address derivation |
| `0x04` | Identity | Solidity ABI encoder |
| `0x05` | ModExp | RSA, ZK circuits |
| `0x06` | BN128Add | zk-SNARK verification |
| `0x07` | BN128Mul | zk-SNARK verification |
| `0x08` | BN128Pairing | Full zk-SNARK proof check |
| `0x09` | BLAKE2F | Zcash, privacy protocols |

---

## Tokenomics

| Parameter | Value |
|-----------|-------|
| Symbol | XNET |
| Hard Cap | 53,000,000 XNET |
| Genesis Premine | 6,000,000 XNET |
| Premine Lock | 2.5 years linear vesting (on-chain) |
| Block Reward | 1.117 XNET → halves every ~4 years |
| Fee Split | 60% Treasury · 15% Grants · 25% Validator |
| Staking APY | 2.5% – 10% (peaks at 50% staking ratio) |
| EVM Chain ID | `2009` |
| SS58 Prefix | `888` |

---

## Benchmarking

```bash
# Build with benchmarking enabled
cargo build --release --features runtime-benchmarks

# Benchmark a pallet
./target/release/xnet-node benchmark pallet \
  --chain dev \
  --pallet pallet_balances \
  --extrinsic "*" \
  --steps 50 \
  --repeat 20 \
  --output pallets/balances/src/weights.rs

# Benchmark block overhead
./target/release/xnet-node benchmark block \
  --chain dev --from 1 --to 100
```

---

## CLI Reference

```
SUBCOMMANDS:
    key             Generate, insert, and inspect keys
    build-spec      Export chain spec to JSON
    check-block     Re-execute and validate blocks
    export-blocks   Dump blocks to binary file
    export-state    Export storage trie as chain-spec patch
    import-blocks   Import blocks from file
    purge-chain     Wipe chain database (irreversible)
    revert          Roll back N blocks
    benchmark       Measure pallet and block weights
    chain-info      Print database metadata
```

---

## Mainnet Launch Checklist

- [ ] Replace `authority_keys_from_seed("Alice")` in `mainnet_config()` with real validator keys
- [ ] Update `FOUNDER_ACCOUNT_ID` to the actual genesis sudo key
- [ ] Remove `invulnerables` list from staking genesis (testnet-only setting)
- [ ] Replace `pallet-sudo` with on-chain governance
- [ ] Commission independent security audit (runtime, EVM, ZKP pallets)
- [ ] Verify vesting: 6,000,000 XNET locked for 13,140,000 blocks (2.5 years)
- [ ] Run `build-spec` and dry-run genesis on a staging environment

> ⚠️ **Never launch mainnet with development keys (Alice/Bob/Charlie seeds).**

---

## License

Licensed under [Apache 2.0](LICENSE).

---

<div align="center">

**Built with Rust & Substrate**

[xnetcoin.org](https://xnetcoin.org) · [t.me/xnethq](https://t.me/xnethq) · [@xnethq](https://x.com/xnethq)

</div>