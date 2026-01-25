# Xnet Blockchain Platform

**Xnet (XNX)** is a high-performance, energy-efficient blockchain built on Substrate, combining Bitcoin's proven economic model with modern consensus technology.

## Overview

Xnet is a pure Proof-of-Stake (PoS) blockchain designed for sustainability, security, and decentralization. With a fixed supply of 53 million tokens, deterministic finality, and 12-second block times, Xnet delivers fast, final transactions with minimal environmental impact.

### Key Characteristics

| Feature | Specification |
|---------|---------------|
| **Consensus** | Nominated Proof-of-Stake (NPoS) with Grandpa Finality |
| **Max Supply** | 53,000,000 XNX tokens |
| **Block Time** | 12 seconds |
| **Finality** | Deterministic (immediate with validator supermajority) |
| **Emission Schedule** | Halving model (Bitcoin-inspired scarcity) |
| **Environment** | Energy-efficient, ~99% less than Proof-of-Work |
| **Network** | Fully decentralized peer-to-peer network |

## Getting Started

### Prerequisites

- **Rust**: 1.70 or later
- **Cargo**: Latest stable version  
- **Git**: For repository management
- **System Requirements**: 
  - 4GB RAM minimum for validator nodes
  - 20GB disk space for chain data
  - Stable internet connection (1Mbps minimum)

### Installation

#### 1. Clone the Repository

```bash
git clone https://github.com/xnetcoin/xnet.git
cd xnet
```

#### 2. Build the Project

```bash
cargo build --release
```

The release binary will be located at: `./target/release/xnet-node`

### Running a Validator Node

To become a validator on the Xnet network:

```bash
./target/release/xnet-node \
  --validator \
  --chain mainnet \
  --name "Your-Validator-Name"
```

### Running a Full Node (Non-Validator)

To run a full node that syncs the chain without validating:

```bash
./target/release/xnet-node \
  --chain mainnet \
  --name "Your-Node-Name"
```

### Testnet Access

To join the testnet for testing and development:

```bash
./target/release/xnet-node \
  --chain testnet \
  --name "Your-Testnet-Node"
```

## Architecture

### Consensus Mechanism

Xnet uses **Nominated Proof-of-Stake (NPoS)** with:

1. **Validators**: Secure the network through block production and finalization
2. **Nominators**: Delegate tokens to validators to earn staking rewards
3. **Collators**: Construct blocks for consensus
4. **Fishermen**: Monitor network for misbehavior

### Finality

Block finality is achieved through **GRANDPA** (GHOST-based Recursive Ancestor Deriving Prefix Agreement):
- Deterministic finality within a single block
- Byzantine fault tolerance for up to 1/3 of validators
- Immediate transaction confirmation

## Token Economics

### Supply Schedule

- **Total Supply**: 53,000,000 XNX (fixed and immutable)
- **Emission Model**: Halving schedule similar to Bitcoin
- **Reward Distribution**: 
  - Validators: Based on staking amount
  - Nominators: Share of validator rewards (commission-based)

### Transaction Fees

Transaction fees are calculated based on:
- Extrinsic size (weight)
- Current network congestion
- Tip amount (optional priority)

## Development

### Project Structure

```
xnet/
├── bin/xnet-node/          # Node implementation
├── client/                 # Client libraries
├── frame/                  # FRAME and runtime pallets
├── primitives/             # Core primitives
└── docs/                   # Documentation
```

### Building from Source

```bash
git clone https://github.com/xnetcoin/xnet.git
cd xnet

# Install dependencies
rustup update stable
rustup target add wasm32-unknown-unknown

# Build
cargo build --release

# Run tests
cargo test --release
```

## Node Configuration

### Important Flags

| Flag | Purpose |
|------|---------|
| `--validator` | Enable validator mode (requires staking) |
| `--chain mainnet` | Connect to mainnet (default) |
| `--chain testnet` | Connect to testnet |
| `--name NODE_NAME` | Set node identifier |
| `--port 30333` | P2P network port |
| `--rpc-port 9933` | RPC server port |
| `--ws-port 9944` | WebSocket RPC port |

## Staking Guide

### Staking Requirements

- **Minimum Bond**: As determined by governance
- **Rewards**: Proportional to your staked amount
- **Slashing Risk**: Potential penalties for malicious behavior

### How to Stake

1. Get tokens from exchanges or community distributions
2. Generate a keypair using `subkey` or hardware wallet
3. Bond tokens via staking transaction
4. Nominate validators to delegate your stake
5. Collect staking rewards every era

## Network Parameters

| Parameter | Value |
|-----------|-------|
| Block Time | 12 seconds |
| Era Duration | 6 hours |
| Maximum Validators | 297 |
| Minimum Commission | 0% |
| Bonding Duration | 28 days |

## Troubleshooting

### Node Won't Sync

```bash
# Check network connectivity
ping one.xnethub.org

# Verify port is open
netstat -an | grep 30333

# Check system time (must be accurate)
timedatectl show
```

### Validator Not Producing Blocks

- Ensure you have minimum bonded tokens
- Verify you're nominated by nominators
- Check session keys are properly configured
- Review logs: `RUST_LOG=debug ./target/release/xnet-node --validator`

## Security Considerations

⚠️ **Important**: 
- Always keep validator keys secure and offline when possible
- Use hardware wallets for large amounts
- Never share seed phrases or private keys
- Validate chain configuration before joining networks
- Keep the node software updated

## Contributing

We welcome contributions from the community! Please:

1. Read [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines
2. Fork the repository
3. Create a feature branch
4. Submit a Pull Request

## License

Xnet is released under dual licensing:
- **GPL-3.0-or-later**: For open-source projects
- **Apache-2.0**: For commercial applications

See [LICENSE-GPL3](LICENSE-GPL3) and [LICENSE-APACHE2](LICENSE-APACHE2) for details.

## Community & Support

- **Website**: [https://xnethub.org](https://xnethub.org)
- **Email**: info@xnethub.org
- **GitHub Issues**: [Submit bug reports](https://github.com/xnetcoin/xnet/issues)
- **Twitter**: [@XnetCoin](https://twitter.com/XnetCoin)

## FAQ

**Q: What is the total supply of XNX tokens?**
A: Fixed at 53,000,000 tokens. No additional tokens will be created.

**Q: How often do I receive staking rewards?**
A: Rewards are distributed every era (approximately 6 hours).

**Q: Can I unstake my tokens anytime?**
A: Yes, but there's a 28-day unbonding period.

**Q: Is Xnet secure?**
A: Xnet uses proven cryptography and Byzantine fault-tolerant consensus.

## Credits

Xnet is built on [Substrate](https://github.com/paritytech/substrate), the blockchain development framework by Parity Technologies.

## Roadmap

- ✅ Testnet Launch (2026 Q1)
- ✅ Mainnet Launch (2026 Q1)
- 🔄 Smart Contracts Support (2026 Q2)
- 🔄 Cross-chain Bridges (2026 Q3)
- 🔄 Governance Integration (2026 Q4)

---

**Version**: 2.0  
**Last Updated**: January 2026  
**Status**: Mainnet Live

For the latest information, visit [https://xnethub.org](https://xnethub.org)
