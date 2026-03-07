# Security Policy

XNET takes security vulnerabilities seriously. We appreciate responsible disclosure and will work promptly to address confirmed issues.

## Supported Versions

| Version | Status |
|---------|--------|
| Latest stable | ✅ Supported |
| Previous minor | ⚠️ Critical fixes only |
| Older versions | ❌ Not supported |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report security issues by email to: **security@xnetprotocol.com**

Your report should include:

- A clear description of the vulnerability
- The affected component(s)
- Steps to reproduce
- Potential impact or attack scenario
- Any suggested mitigations (optional)

You will receive an acknowledgment within **two business days**. We aim to deliver a fix or mitigation within **14 days** for critical issues, depending on complexity.

We ask that you:

- Give us reasonable time to investigate and fix the issue before disclosing it publicly
- Avoid accessing, modifying, or deleting user data as part of research
- Not conduct denial-of-service attacks or disrupt live network services

## Responsible Disclosure

If you believe your finding is eligible for a bug bounty, note that clearly in your report. Bounty eligibility and amounts are assessed case by case based on severity and exploitability.

## Threat Model Scope

The following are considered **in scope**:

- XNET runtime (pallets, state transitions)
- Node client (RPC, consensus, networking)
- Cryptographic primitives and key handling
- Chain specification and genesis configuration

The following are considered **out of scope**:

- Third-party wallets or interfaces not maintained by this project
- Social engineering attacks
- Issues in upstream dependencies (report those upstream)
- Theoretical attacks with no viable exploit path

## On-Chain Emergency Response

If a vulnerability requires an emergency runtime upgrade, the governance process may be bypassed temporarily via the sudo key. This action will be announced publicly immediately. The sudo key will be removed once the network reaches operational maturity.
