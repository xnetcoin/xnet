# Contributing to XNET

XNET is an open-source blockchain project and welcomes contributions from the community. This document outlines the expectations and procedures for contributing code, documentation, and other improvements.

Join the discussion on our community channels:

- Telegram: [t.me/xnethq](https://t.me/xnethq)
- GitHub Issues: for bug reports and feature proposals

## What We Build

XNET is a Nominated Proof-of-Stake blockchain built on the Substrate framework. Contributions may include:

- Runtime pallets and configuration changes
- Node client improvements
- Tooling, scripts, and developer experience
- Documentation and specifications

## Ground Rules

All contributors, including maintainers, must follow these rules:

1. **No force pushes** to the main branch or any shared branch. Rebase only in your own fork.
2. **Branch naming**: use short descriptive prefixes, e.g. `fix/staking-reward`, `feat/evm-support`.
3. **All changes must go through a pull request.** Direct commits to `main` are not permitted.
4. **CI must pass** before a pull request can be merged.
5. Follow the [Style Guide](STYLE_GUIDE.md) for all Rust code.

## Pull Request Process

1. Fork the repository and create a branch from `main`.
2. Write clear, focused commits. Each commit should represent a single logical change.
3. Open a pull request with a description that covers:
   - What the change does
   - Why it is needed
   - How it was implemented and what it affects
   - Any migrations, API breaks, or dependency changes
4. Tag the PR appropriately:
   - `bug` — fixes a defect
   - `feature` — adds new functionality
   - `docs` — documentation only
   - `breaking` — breaks an existing API or on-chain interface
   - `runtime-migration` — requires a storage migration
5. At least one review approval is required before merging.
6. Do not merge your own pull request unless no other maintainer is available and urgency is justified.

## Runtime Migrations

Pull requests that include runtime storage migrations must:

- Be tagged with `runtime-migration`
- Include a `try-runtime` test verifying migration correctness
- Document the migration in the PR description with before/after state layout

## Breaking Changes

If your change modifies an external API, RPC interface, or runtime dispatchable:

- Tag the PR with `breaking`
- Describe in the PR description what changed and how to adapt existing integrations
- Update relevant documentation

## Review Standards

Reviewers should decline approval if the change:

1. Introduces bugs or incorrect on-chain behavior
2. Adds undue complexity without clear benefit
3. Breaks the coding style or violates safety guidelines
4. Degrades performance without justification
5. Removes functionality without agreement from maintainers

Reviews should not be used to block changes solely because of stylistic preference or minor disagreements about implementation approach.

## Issues

Label issues with:

- `I-bug` — confirmed defect
- `I-question` — discussion or clarification needed
- `Z-easy` / `Z-medium` / `Z-hard` — difficulty estimation

## Releases

Releases are managed by the core maintainers. Version numbers follow [Semantic Versioning](https://semver.org/). Runtime spec versions are incremented on every on-chain change.

## Changes to This Document

This document may be updated by contributors through pull requests. Significant changes to the contribution process should be announced in community channels.
