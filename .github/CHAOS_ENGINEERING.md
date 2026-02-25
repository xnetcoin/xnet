# Network Resilience & Chaos Engineering

## Overview

XNet validators must be able to handle network failures, Byzantine attacks, and chaos scenarios gracefully.

## Chaos Scenarios Tested

### 1. Network Partition (Split Brain)
- Network splits into isolated groups
- Validators in each partition try to reach consensus independently
- Expected: System should heal when partition heals
- **Test runs**: Daily
- **Duration**: 15 minutes

### 2. Validator Crash & Recovery
- Validators randomly crash
- System continues (requires >2/3 healthy validators)
- Crashed validators catch up from state sync
- **Test runs**: Daily
- **Duration**: 10 minutes

### 3. Transaction Pool Stress
- 10,000+ transactions submitted simultaneously
- Tests memory usage and CPU load
- Validates garbage collection
- **Test runs**: Daily
- **Duration**: 5 minutes

### 4. Byzantine Behavior
- Validators propose conflicting blocks
- System rejects invalid proposals
- Consensus remains intact
- **Frequency**: Weekly

## Multi-Node Testing

Every PR that touches consensus code automatically:

1. Creates a 4-node validator network
2. Verifies block production and finality
3. Tests Byzantine fault tolerance (BFT)
4. Measures time to consensus

```bash
# Run locally
./scripts/test-multi-node.sh --nodes 4 --duration 300
```

## Performance Benchmarks

**Block production time**: < 12 seconds
**Finality time**: < 60 seconds
**Sync speed**: > 100 blocks/second

View benchmark results on each PR.

## Local Chaos Testing

```bash
# Install chaos tools
cargo install zombienet

# Run multi-node devnet with chaos
zombienet spawn config.toml
```

## Failure Recovery Expectations

- **Validator crash**: 30 second recovery
- **Network partition**: Immediate from partition detection
- **100 block fork**: Resolved within 1 block
- **50% validator loss**: System continues (requires restart)

## Monitoring in Production

Enable chaos metrics:
```bash
xnet-node --validator --enable-chaos-metrics
```

Metrics available at: `http://localhost:9944/metrics`

Key metrics:
- `xnet_consensus_round_time`
- `xnet_block_production_time`
- `xnet_network_latency`
- `xnet_peer_count`

## Resources

- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Substrate Network Simulation](https://github.com/polkadot-sdk/polkadot-sdk/tree/master/zombienet)
- [Byzantine Fault Tolerance](https://en.wikipedia.org/wiki/Byzantine_fault)
