# Ink! Smart Contract Tests

XNet uses **ink!** (Rust-based smart contracts) instead of Solidity.

## Running Tests

```bash
# Run all contract tests
cargo test -p pallet-contracts

# Run specific test
cargo test -p pallet-contracts test_transfer_preserves_total_supply

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p pallet-contracts

# Run benchmarks
cargo test -p pallet-contracts --benches
```

## Test Coverage

```bash
# Generate coverage report
cargo tarpaulin -p pallet-contracts --out Html --output-dir coverage
```

## Test Structure

Tests are organized in `src/tests_unit.rs`:

- **Transfer Tests**: Verify token transfers work correctly
- **Allowance Tests**: Test approval and transfer_from
- **Supply Tests**: Verify mint/burn affect total supply
- **Balance Tests**: Check balance updates
- **Edge Cases**: Integer overflow, zero transfers, etc.

## Property-Based Testing

For more advanced testing, use `proptest`:

```toml
[dev-dependencies]
proptest = "1.0"
```

Then write property tests:

```rust
#[test]
fn prop_transfer_preserves_supply() {
    proptest!(|(amount in 0u128..1000) | {
        // Your property test
    });
}
```

## Fuzzing

Enable fuzzing with `cargo-fuzz`:

```bash
cargo install cargo-fuzz
cargo fuzz
```

Create fuzz targets in `fuzz/fuzz_targets/`.
