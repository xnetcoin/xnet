## Description
Provide a comprehensive description of the changes introduced by this PR. Link to any relevant issues using `Fixes #...` or `Resolves #...`.

---

## Type of change

Please delete options that are not relevant.

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Runtime Upgrade / Storage Migration
- [ ] Documentation update

---

## Substrate / Node Requirements Checklist

Given that XNet is a Rust-based Substrate chain actively developing EVM, Wasm/ink!, and ZKP features, all PRs touching the core logic **must** adhere to the following checklist:

- [ ] I have read the [CONTRIBUTING.md](../CONTRIBUTING.md) guidelines.
- [ ] My code strictly follows `rustfmt` formatting standards (`cargo fmt --all`).
- [ ] My code passes all `cargo clippy` linting requirements without warnings.
- [ ] I have added/updated tests that prove my fix is effective or that my feature works.
- [ ] **Runtime Only:** I have appropriately benchmarked any new extrinsics and updated corresponding Weight files.
- [ ] **Runtime Only:** If this PR modifies storage structures, I have included the necessary storage-migration logic.
- [ ] I have fully documented new structs/functions using `rustdoc`.

---

## Notes for Reviewers
(Any specific areas the reviewer should focus on? E.g., EVM precompile logic, consensus configuration, ink! contract dependencies).
