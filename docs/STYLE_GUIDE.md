---
title: Rust Style Guide — XNET
---

All Rust code in this repository must pass `cargo fmt` before submission. The
settings in `rustfmt.toml` enforce most of the rules below automatically.
What follows covers the cases `rustfmt` cannot handle.

---

## Formatting

- **Indentation**: tabs, not spaces.
- **Line length**: 100 characters. Tabs count as 4 characters. Exceeding 100
  characters is allowed only in extraordinary circumstances; 120 is a hard ceiling.
- **Nesting depth**: aim for ≤ 5 levels. At 6 or deeper, refactor into helper
  functions or `let` bindings to flatten the shape.
- No trailing whitespace on any line.
- Follow-on (continuation) lines use a single extra indent level — not two.

```rust
fn calculate(a: i64, b: i64) -> bool {
	let result = a * b
		- b / a
		+ sqrt(a)
		- sqrt(b);
	result > 10
}
```

- When a function call or parameter list breaks across lines, every parameter
  goes on its own line, and the closing delimiter is on its own line:

```rust
// ✓ correct
fn init(
	config: Config,
	storage: StorageRef,
	network: NetworkHandle,
) -> Result<Node> {
	…
}

// ✗ wrong — mixed inline and wrapped params
fn init(config: Config, storage: StorageRef,
	network: NetworkHandle) -> Result<Node> { … }
```

- `where` clauses are indented one level; their items are indented one further.
- Always include a trailing comma on the last item of any multi-line
  comma-delimited list (struct fields, enum variants, function parameters, match
  arms in block form) when the language allows it.
- Single-line comma-delimited lists do **not** have a trailing comma.
- Avoid unnecessary semicolons at the end of block expressions used as return
  values.

---

## Naming

Follow [Rust API guidelines](https://rust-lang.github.io/api-guidelines/):

- Types, traits, enum variants: `UpperCamelCase`
- Functions, methods, variables, modules: `snake_case`
- Constants and statics: `SCREAMING_SNAKE_CASE`
- Type parameters: short `UpperCamelCase`, e.g. `T`, `E`, `AccountId`

Pallet storage items follow on-chain naming conventions and are exempt from the
`snake_case` function rule (they are types, not functions).

---

## Safety and Panics

- **No `unwrap()`** outside of test code. Use `expect("reason; qed")` when you
  can prove statically that an `Option` or `Result` is always `Ok`/`Some`, and
  document why:

```rust
let path = self.path().expect(
	"DiskDirectory always returns a path; \
	 this variant cannot have no path; qed"
);
```

- **No raw `panic!`** in runtime code. Runtime panics produce invalid blocks.
  Return `Err(…)` or use `ensure!` instead.

- **Unsafe code** requires justification in a `// SAFETY:` comment immediately
  before the `unsafe` block. Before introducing unsafe, evaluate:
  - How much is actually gained in performance or ergonomics?
  - How likely is it that the invariant could be violated in future?
  - Are there tests or tools that would catch a violation early?

---

## Documentation

- All `pub` items in crates that are part of the external API must have doc
  comments (`///`).
- Module-level documentation uses `//!`.
- The first sentence of every doc comment must stand alone as a complete
  description — it appears in the generated index tables.
- Use `[`item`]` intra-doc links rather than bare backticks when referring to
  other items.
- Code examples in doc comments are compiled and run as integration tests;
  keep them correct and up to date.
- Internal implementation notes that are not relevant to callers belong in
  `//` comments, not `///`.

---

## Cargo.toml Formatting

Feature lists are formatted by `zepter format features`. The rules are:

- Single-entry features fit on one line: `default = ["std"]`
- Multi-entry features are broken to one entry per line with a trailing comma:

```toml
[features]
default = [
	"pallet-balances/std",
	"pallet-staking/std",
	"sp-runtime/std",
]
```

All entries within a section are sorted alphabetically.

---

## Match Arms

Each match arm is either a single expression followed by a comma, or a braced
block. Do not combine both styles on the same arm:

```rust
match action {
	Action::Transfer => transfer(),
	Action::Stake => { validate(); stake() },
//	Action::Exit => { return Err(…) }   // ✗ — trailing comma missing
	Action::Exit => return Err(Error::NotAllowed),
}
```

Blocks are only used when multiple statements are genuinely required.

---

## Error Handling

- Runtime dispatchables return `DispatchResult` or `DispatchResultWithPostInfo`.
  Use `ensure!(condition, Error::<T>::Variant)` for guard conditions.
- Off-chain code (RPC, CLI) uses `anyhow` or explicit `Result` chains.
- Never silently discard errors with `let _ = fallible_call();` without a
  comment explaining why the result is intentionally ignored.
