# XnetXCoin (XNX) - Substrate Blockchain

## Overview

XnetXCoin is a Proof-of-Stake (PoS) cryptocurrency built on Substrate with a Bitcoin-like halving emission schedule.

## Tokenomics

| Property | Value |
|----------|-------|
| **Token Symbol** | XNX |
| **Decimals** | 18 |
| **Max Supply** | 53,000,000 XNX |
| **Premine** | 6,000,000 XNX |
| **Mining Supply** | 47,000,000 XNX |
| **Block Time** | 12 seconds |
| **Initial Block Reward** | 1.565 XNX |
| **Halving Interval** | 4 years (~10,512,000 blocks) |

## Emission Schedule

| Era | Years | Block Reward | Total Minted | Cumulative % |
|-----|-------|--------------|--------------|--------------|
| 1 | 0-4 | 1.565 XNX | ~16.45M | 35% |
| 2 | 4-8 | 0.7825 XNX | ~8.22M | 52.5% |
| 3 | 8-12 | 0.391 XNX | ~4.11M | 61.25% |
| 4 | 12-16 | 0.195 XNX | ~2.05M | 65.6% |
| ... | ... | /2 each era | ... | ... |

## Consensus

- **Block Production**: Aura (Authority-based Round-robin)
- **Finality**: GRANDPA (GHOST-based Recursive Ancestor Deriving Prefix Agreement)
- **Staking**: NPoS (Nominated Proof-of-Stake)

## Staking Parameters

| Parameter | Value |
|-----------|-------|
| **Min Validator Bond** | 10,000 XNX |
| **Min Nominator Bond** | 1,000 XNX |
| **Session Length** | 1 hour (300 blocks) |
| **Sessions Per Era** | 6 (~6 hours) |
| **Bonding Duration** | 28 eras (~1 week) |
| **Slash Defer Duration** | 27 eras |
| **Max Validators** | 100 |
| **Max Nominators Per Validator** | 256 |

## Premine Wallet

The 6,000,000 XNX premine is allocated to the founder wallet:
```
5G1YRg4aKtHemtpuxh15u3YCBJwYBWt8ykeGbqhxDGuJTNXQ
```

## Building

### Prerequisites

- Rust 1.70+ with nightly toolchain
- WASM target: `rustup target add wasm32-unknown-unknown`
- 8GB+ RAM recommended

### Build Commands

```bash
# Navigate to node-template directory
cd substrate/bin/node-template

# Build in release mode
cargo build --release

# Build with features
cargo build --release --features runtime-benchmarks
```

### Running

```bash
# Run development node
./target/release/node-template --dev

# Run mainnet node
./target/release/node-template --chain mainnet

# Run testnet node
./target/release/node-template --chain testnet
```

## Project Structure

```
bin/node-template/
├── node/
│   └── src/
│       ├── chain_spec.rs      # Genesis configurations
│       ├── command.rs         # CLI commands
│       └── service.rs         # Node service
├── pallets/
│   ├── template/              # Example pallet
│   └── block-rewards/         # XNX block rewards with halving
│       └── src/lib.rs
├── runtime/
│   └── src/lib.rs             # Runtime configuration
└── XNETXCOIN.md               # This file
```

## Custom Pallets

### Block Rewards Pallet (`pallet-block-rewards`)

Implements the XnetXCoin emission schedule with Bitcoin-like halving:

- **Storage**:
  - `TotalMinted`: Total XNX minted through block rewards
  - `CurrentEra`: Current halving era (0, 1, 2, ...)

- **Events**:
  - `BlockRewardIssued`: Emitted when a validator receives a block reward
  - `HalvingOccurred`: Emitted when a halving event occurs
  - `MaxEmissionReached`: Emitted when max emission (47M XNX) is reached

- **Functions**:
  - `calculate_current_reward()`: Calculate reward based on current block
  - `get_total_minted()`: Get total XNX minted
  - `get_remaining_emission()`: Get remaining emission capacity

## Chain Specifications

### Development (`--dev`)
- Single validator (Alice)
- All accounts pre-funded for testing

### Testnet (`--chain testnet`)
- Two validators (Alice, Bob)
- Pre-funded test accounts

### Mainnet (`--chain mainnet`)
- Real validator keys required
- Founder receives 6M XNX premine
- Staking enabled

## API Endpoints

Default ports:
- HTTP RPC: 9944
- WebSocket: 9944
- P2P: 30333

## License

GPL-3.0 / Apache-2.0

---

**XnetXCoin** - A fair, halving-based PoS cryptocurrency.
