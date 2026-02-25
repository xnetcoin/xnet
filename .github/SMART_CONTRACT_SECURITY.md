# Smart Contract Security Testing Guide

## Overview

XNet uses **ink!** - Rust-based smart contracts for the blockchain runtime. This provides memory safety and compile-time verification advantages over Solidity.

## 1. Unit Testing

### Running Tests

```bash
# Run all contract unit tests
cargo test -p pallet-contracts --lib

# Run specific test
cargo test -p pallet-contracts test_transfer_preserves_total_supply

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p pallet-contracts --lib

# Show test output
cargo test -p pallet-contracts -- --nocapture
```

### Test Location
Tests are in: `frame/contracts/src/tests_unit.rs`

Tests cover:
- Transfer operations
- Balance updates
- Allowance mechanisms
- Mint/Burn operations
- Edge cases (overflow, underflow, insufficient balance)

## 2. Code Quality Checks

### Clippy (Linter)
```bash
cargo clippy -p pallet-contracts -- -D warnings
```

Checks for:
- Common mistakes
- Performance issues
- Idiomatic Rust patterns
- Possible logic errors

### Formatting
```bash
cargo fmt -p pallet-contracts -- --check
```

### Unsafe Code Detection

```bash
# Scan for unsafe blocks
grep -r "unsafe {" frame/contracts/src --include="*.rs"

# Scan for unwrap() calls
grep -r "\.unwrap()" frame/contracts/src --include="*.rs"

# Scan for panic patterns
grep -r "panic!\|todo!\|unimplemented!" frame/contracts/src --include="*.rs"
```

## 3. Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cd frame/contracts
cargo tarpaulin -p pallet-contracts --out Html --output-dir coverage

# View results
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

**Coverage Requirements:**
- Minimum 80% code coverage
- Critical paths must have 95%+ coverage

## 4. Undefined Behavior Detection

Using Miri (experimental):

```bash
cargo +nightly miri test -p pallet-contracts --lib
```

Detects:
- Use-after-free
- Invalid memory access
- Data races
- Undefined behavior patterns

## 5. Dependency Security

```bash
# Audit dependencies for CVEs
cargo audit

# Check specific package
cargo audit --package <package-name>
```

All dependencies must have no critical vulnerabilities.

## 6. Integration Testing

Integration tests verify contract interactions:

```bash
cargo test -p pallet-contracts --test '*'
```

Tests are located in: `frame/contracts/test/integration_tests.rs`

## 7. Documentation

Ensure all public functions have documentation:

```bash
cargo doc -p pallet-contracts --no-deps
```

Fail on warnings:
```bash
RUSTDOCFLAGS="-D warnings" cargo doc -p pallet-contracts --no-deps
```

## Security Best Practices

### 1. Avoid Unsafe Code
- Use safe Rust whenever possible
- Document why unsafe is necessary
- Prove correctness with tests

### 2. Use Result Types
```rust
// ❌ Bad
let balance = get_balance(user).unwrap();

// ✅ Good
match get_balance(user) {
    Ok(balance) => { /* ... */ },
    Err(e) => { /* handle error */ }
}
```

### 3. Check Arithmetic
```rust
// ❌ Bad - can overflow
let result = amount + fee;

// ✅ Good - safe arithmetic
let result = amount.checked_add(fee)?;
```

### 4. Validate Inputs
```rust
// ✅ Always validate
if amount == 0 {
    return Err("Amount must be > 0");
}
```

### 5. Prevent Reentrancy
Rust's ownership system prevents most reentrancy issues automatically.

### 6. Emit Events
```rust
// ✅ Always emit events for important operations
self.env().emit_event(Transfer {
    from: Some(from),
    to: Some(to),
    value,
});
```

## CI/CD Checks

All the above are automatically run on:
- **Push** to main/develop branches
- **Pull Requests** to main/develop branches
- **Daily** scheduled runs

See `.github/workflows/smart-contract-security.yml` for details.

## Emergency Response

If a vulnerability is found:

1. **Don't commit or push**
2. **Email security@xnetcoin.org** immediately
3. **Don't open public issues**
4. XNet team will coordinate responsible disclosure

## Resources

- [ink! Documentation](https://docs.rs/ink/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [OWASP Smart Contract Top 10](https://owasp.org/www-community/vulnerabilities/)
- [Substrate Runtime Security](https://substrate.dev/)

