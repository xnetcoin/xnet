# Devnet Testing Guide

## Ephemeral Devnet Workflow

Every PR that modifies core code automatically:

1. **Creates** a temporary 3-node testnet
2. **Deploys** smart contracts from the PR
3. **Runs** integration tests
4. **Benchmarks** performance
5. **Cleans up** after completion

This happens automatically - no manual setup needed!

## What Gets Tested

### Contract Deployment
- All smart contracts are deployed to the devnet
- Contract state is initialized
- Events are verified

### Integration Tests
- End-to-end transaction flows
- Cross-contract interactions
- State consistency

### Performance
- Block production time
- Transaction throughput
- Memory usage

### Network Health
- All 3 nodes sync properly
- Blocks finalize
- No peer disconnections

## Accessing Devnet During PR

The RPC endpoint is available at:
```
http://localhost:9944
```

Available while PR is being tested (60 minutes max).

## Manual Devnet Testing

Start a local 3-node devnet:

```bash
docker-compose -f docker/docker-compose.yml up
```

Access nodes:
- Alice: http://localhost:9944
- Bob: http://localhost:9945
- Charlie: http://localhost:9946

### Submit test transaction:
```bash
curl -X POST http://localhost:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"author_submitExtrinsic",
    "params":["0x..."],
    "id":1
  }'
```

### Check consensus:
```bash
curl -X POST http://localhost:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"system_health",
    "params":[],
    "id":1
  }'
```

## Devnet Configuration

Default setup:
- **Nodes**: 3 validators
- **Block time**: 12 seconds
- **Finality**: 60 seconds
- **Network**: Private (no mainnet connection)

## Troubleshooting

**Node won't start**:
```bash
docker logs xnet-alice
```

**Transaction not included**:
```bash
# Check mempool
curl -X POST http://localhost:9944 \
  -d '{"jsonrpc":"2.0","method":"author_pendingExtrinsics","params":[],"id":1}'
```

**Nodes out of sync**:
```bash
# Restart devnet
docker-compose -f docker/docker-compose.yml down
docker-compose -f docker/docker-compose.yml up
```

## Cleanup

```bash
docker-compose -f docker/docker-compose.yml down -v
```

This removes all containers and persistent data.
