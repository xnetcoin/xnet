<div align="center">

# XNET Protocol

### Independent Layer 1 Blockchain

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust)](https://rustup.rs)
[![Substrate](https://img.shields.io/badge/Substrate-Framework-brightgreen?style=flat-square)](https://substrate.io)
[![EVM](https://img.shields.io/badge/EVM-Chain%20ID%202009-blue?style=flat-square&logo=ethereum)](https://xnetcoin.org)
[![ink!](https://img.shields.io/badge/ink!-WASM%20Contracts-purple?style=flat-square)](https://use.ink)
[![ZKP](https://img.shields.io/badge/ZKP-Dual%20System-red?style=flat-square)](https://xnetcoin.org)
[![License](https://img.shields.io/badge/License-GPL--3.0-lightgrey?style=flat-square)](LICENSE)

**[Website](https://xnetcoin.org) · [Telegram](https://t.me/xnethq) · [Twitter/X](https://x.com/xnethq) · [Docs](https://docs.xnetcoin.org) · [Explorer](https://explorer.xnetcoin.org)**

</div>

---

## Overview

XNET is an independent Layer 1 blockchain built from scratch on Substrate. Not a fork. Not a parachain. Not a template — every line of runtime code was written from the ground up.

Most blockchains force a choice: EVM or WASM, fast or private, programmable or sound money. XNET doesn't make you choose.

| Feature | Description | Status |
|---|---|---|
| **EVM Compatible** | Deploy any Solidity contract. MetaMask, Hardhat, Foundry work out of the box | ✅ Live |
| **ink! / WASM** | Rust-based smart contracts with superior safety and performance | ✅ Live |
| **ZK Layer 1** | EVM-based Solidity ZK verifier contracts | ✅ Live |
| **ZK Layer 2** | `pallet-zk-verifier` — native Groth16 in the runtime | 🔧 Final integration |
| **XCM Interop** | Trustless cross-chain messaging | 📋 Testnet phase |
| **Sound Money** | 53,000,000 XNC hard cap. Bitcoin-style halving | ✅ Encoded |

---

## Quick Start

> New to Rust? See [docs/rust-setup.md](docs/rust-setup.md) for full setup instructions.

```bash
# Clone
git clone https://github.com/xnetcoin/xnet
cd xnet

# Build
cargo build --release

# Run local dev node
./target/release/xnet-node --dev
```

Connect at `ws://127.0.0.1:9944` via [Polkadot.js Apps](https://polkadot.js.org/apps).

---

## Repository Structure

```
xnet/
├── bin/                        # Node binary entrypoint
├── contracts/                  # Example smart contracts (EVM + ink!)
├── docker/                     # Docker & docker-compose configs
├── docs/                       # Documentation
│   ├── rust-setup.md           # Environment setup guide
│   └── ...
├── pallets/                    # Custom FRAME pallets
│   ├── block-rewards/          # Bitcoin-style halving block rewards
│   └── zk-verifier/            # On-chain Groth16 ZK-SNARK verification
├── primitives/                 # Shared types and traits
├── runtime/                    # WASM runtime
│   ├── src/
│   │   ├── lib.rs              # Runtime construction — all pallets wired here
│   │   └── precompiles.rs      # EVM precompile set (0x01–0x09)
│   └── build.rs                # Compiles runtime to WASM blob
├── scripts/                    # Utility and deployment scripts
├── src/                        # Node source (cli, service, rpc, chain_spec)
├── test/                       # Integration tests
├── test-utils/                 # Shared test utilities
├── xnet-privacy-contracts/     # Privacy contract examples
└── zombienet/                  # Multi-node test network configs
```

---

## Running a Node

### Development — single node, no peers needed

```bash
./target/release/xnet-node --dev

# With persistent state
./target/release/xnet-node --dev --base-path /tmp/xnet-dev
```

### Local Testnet — 3 validators

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

### Docker

```bash
docker-compose -f docker/docker-compose.yml up
```

---

## Connecting Wallets

### Polkadot.js Apps

1. Open [polkadot.js.org/apps](https://polkadot.js.org/apps)
2. Settings → Custom endpoint → `ws://127.0.0.1:9944`

### MetaMask / EVM Wallets

| Field | Value |
|---|---|
| Network Name | XNET |
| RPC URL | `http://127.0.0.1:9944` |
| Chain ID | `2009` |
| Symbol | `XNC` |
| Explorer | `https://explorer.xnetcoin.org` |

---

## Smart Contracts

### Solidity (EVM)

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

### ink! (WASM)

```bash
cargo install cargo-contract --force

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

Bitcoin-style emission with halving every ~4 years:

| Parameter | Value |
|---|---|
| Initial reward | 1,117 XNC / block |
| Halving interval | 21,038,400 blocks (~4 years) |
| Hard cap | 53,000,000 XNC |

### `pallet-zk-verifier`

Native Groth16 ZK-SNARK verification in the Substrate runtime — written entirely in Rust, no external cryptographic library dependencies.

- Full Groth16 verifier — supports any Circom-compiled circuit
- Nullifier registry — prevents proof replay attacks
- Block-level proof limits — DoS protection
- SnarkJS compatible

No equivalent open-source Groth16 pallet exists in the Substrate ecosystem.

```bash
# Generate proof with Circom + SnarkJS
circom circuit.circom --r1cs --wasm --sym
snarkjs groth16 setup circuit.r1cs pot12_final.ptau circuit_final.zkey
snarkjs groth16 prove circuit_final.zkey witness.wtns proof.json public.json

# Submit proof on-chain
# pallet_zk_verifier::verify_proof(proof, public_inputs, verification_key)
```

---

## EVM Precompiles

| Address | Name | Used By |
|---|---|---|
| `0x01` | ECRecover | Wallets, signature verification |
| `0x02` | SHA-256 | Bitcoin bridge, hashing |
| `0x03` | RIPEMD-160 | Bitcoin address derivation |
| `0x04` | Identity | Solidity ABI encoder |
| `0x05` | ModExp | RSA, ZK circuits |
| `0x06` | BN128Add | ZK-SNARK verification |
| `0x07` | BN128Mul | ZK-SNARK verification |
| `0x08` | BN128Pairing | Full ZK-SNARK proof check |
| `0x09` | BLAKE2F | Privacy protocols |

---

## Tokenomics

| Parameter | Value |
|---|---|
| Token Name | XNET |
| Ticker | XNC |
| Hard Cap | 53,000,000 XNC |
| Genesis Premine | 6,000,000 XNC — all locked, 0 liquid at genesis |
| Block Reward | 1,117 XNC → halves every ~4 years |
| Fee Distribution | 60% Treasury · 15% Dev Grants · 25% Validators |
| Validator Min Bond | 8,000 XNC |
| Nominator Min Bond | 1,000 XNC |
| EVM Chain ID | `2009` |
| SS58 Prefix | `888` |

### Supply Schedule

```
Total: 53,000,000 XNC
├── Genesis Premine    6,000,000 XNC  (all locked at genesis)
│   ├── Founder              1,000,000  — 2.5 years vesting
│   ├── Investor Reserve     1,000,000  — 2.5 years vesting
│   ├── Ecosystem Fund       2,000,000  — 3 years vesting
│   │   └── Developer Grants 1,000,000  — governance-released
│   └── Treasury Reserve     2,000,000  — governance-controlled
└── Block Rewards     47,000,000 XNC
    ├── Era 1 (Year 0–4)    1,117 XNC/block  → ~23.5M XNC
    ├── Era 2 (Year 4–8)    ~558 XNC/block   → ~11.75M XNC
    ├── Era 3 (Year 8–12)   ~279 XNC/block   → ~5.87M XNC
    └── Era 4+ ... until hard cap (~100 years)
```

---

## Technical Specs

| Parameter | Value |
|---|---|
| Consensus | BABE + GRANDPA |
| Staking | NPoS — Sequential Phragmén |
| Block Time | 6 seconds |
| Finality | ~12 seconds |
| Block Size | 5 MB |
| Gas Limit | 75,000,000 / block |
| Era Duration | 24 hours (1,800 blocks) |
| Unbonding Period | 28 eras (~28 days) |
| Max Validators | 100 |

---

## Roadmap

```
[NOW]   pallet-zk-verifier — native Groth16, final integration
  ↓
[NEXT]  Public testnet launch
  ↓
[THEN]  XCM — cross-chain messaging (during testnet)
  ↓
[THEN]  Sudo removal → on-chain governance
  ↓
[THEN]  Mainnet
  ↓
[THEN]  DEX · Bridge · CEX listings · Mobile wallet
```

| Phase | Status | Includes |
|---|---|---|
| Phase 1 — Foundation | ✅ Complete | Runtime, EVM, ink!, NPoS, halving, treasury |
| Phase 2 — ZKP | 🔧 In Progress | pallet-zk-verifier final integration |
| Phase 3 — Testnet + XCM | 🔜 Next | Public testnet, XCM, explorer, wallet, docs |
| Phase 4 — Mainnet | 🔜 After testnet | Governance, mainnet genesis |
| Phase 5 — Ecosystem | 🔜 Post-mainnet | DEX, bridge, CEX, mobile wallet |

---

## Network Status

| Network | Status | RPC | Explorer |
|---|---|---|---|
| Mainnet | 🔜 Coming Soon | — | — |
| Testnet | 🔜 Launching Soon | — | — |
| Local Dev | ✅ Available | `ws://127.0.0.1:9944` | — |

---

## Benchmarking

```bash
cargo build --release --features runtime-benchmarks

./target/release/xnet-node benchmark pallet \
  --chain dev \
  --pallet pallet_zk_verifier \
  --extrinsic "*" \
  --steps 50 \
  --repeat 20 \
  --output pallets/zk-verifier/src/weights.rs
```

---

## Developer Grants

**1,000,000 XNC** from the ecosystem fund plus **15% of all transaction fees** are allocated to developer grants. The formal grant program launches with testnet — but we're already listening.

If you have an idea for something you want to build on XNET — a DeFi protocol, an NFT platform, a ZK application, a wallet, an explorer, developer tooling, or anything else that would make this ecosystem stronger — reach out now. Early contributors will be remembered when the grants program goes live.

You don't need a finished product. A clear idea is enough to start a conversation.

**What we're looking for:**
- DeFi — DEXs, lending, stablecoins, yield protocols
- ZK applications — privacy tools, identity, provable computation
- NFT infrastructure — minting, marketplaces, provenance
- Developer tooling — SDKs, indexers, testing frameworks
- Infrastructure — RPC providers, explorers, monitoring
- Anything that makes XNET more useful for developers or users

**Get in touch — whichever works for you:**

→ Telegram: [t.me/xnethq](https://t.me/xnethq)
→ Twitter / X: [x.com/xnethq](https://x.com/xnethq)
→ Email: [xnetprotocol@gmail.com](mailto:xnetprotocol@gmail.com)

---

## Contributing

```bash
git clone https://github.com/xnetcoin/xnet
git checkout -b feat/your-feature
cargo test
# open a pull request
```

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a PR.

---

## Security

Found a vulnerability? Report it privately via [security@xnetcoin.org](mailto:security@xnetcoin.org) or see [SECURITY.md](SECURITY.md).

Do not open a public GitHub issue for security vulnerabilities.

---

## License

- [GPL-3.0](LICENSE-GPL3) — node and runtime
- [Apache-2.0](LICENSE-APACHE2) — pallets and libraries

---

<div align="center">

Built with Rust & Substrate — from scratch.

**[xnetcoin.org](https://xnetcoin.org) · [t.me/xnethq](https://t.me/xnethq) · [@xnethq](https://x.com/xnethq)**

</div>

---

*A note on these docs: English isn't my first language. The last twelve months went into the code, not into writing. I used AI to help put these docs together — to make sure the technical details were presented clearly and nothing important was left out. The architecture, the pallets, the late nights — that's all mine. The words got some help. If you want to know what this project really is, read the commits.*