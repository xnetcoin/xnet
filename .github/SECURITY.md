# Security Policy

> ⚠️ **Current Status:** XNet is currently in its **Testnet phase** and undergoing active, heavy development. Core network components such as consensus, Wasm/ink! execution, EVM interoperability, and upcoming features (ZKP, XCM) are continuously being updated.

Security is our absolute highest priority. As a decentralized infrastructure provider, the integrity of the XNet protocol and the safety of our users' assets are paramount. We appreciate the efforts of the security community in helping us find and fix vulnerabilities.

## Reporting a Vulnerability

**DO NOT OPEN PUBLIC ISSUES FOR SECURITY VULNERABILITIES.**

If you discover a potential vulnerability in the XNet core node, runtime pallets, EVM integration, or Wasm/ink! components, please report it directly and privately to our security team:

*   **Email:** security@xnetcoin.org
*   **Telegram:** Reach out directly to the core team admins in our [Official Telegram (xnethq)](https://t.me/xnethq) for a secure communication channel.

### What to Include in Your Report

To help us triage and resolve issues quickly, please include:
1.  **Summary:** A brief description of the vulnerability.
2.  **Severity:** Your assessment of the impact (e.g., node crash, unauthorized fund transfer, consensus failure).
3.  **Reproduction Steps:** Detailed, step-by-step instructions (or an exploit script/POC) to reproduce the issue.
4.  **Environment details:** Branch, commit hash, and operating system used.
5.  **Suggested Mitigation:** If you have an idea of how to fix it, let us know!

## Response Timeline

*   **Acknowledgment:** Within 48 hours of receipt.
*   **Initial Assessment:** Within 1 week.
*   **Patch Development:** Commences immediately upon verification for critical issues. 

## Bug Bounty Program

Because we are currently in **Testnet** stage, a formal Mainnet bug bounty program with defined tier rewards is not yet open. 

However, we deeply value the contributions of white-hat hackers and security researchers. Significant runtime, EVM, Wasm, or consensus-level vulnerabilities reported during the Testnet phase may be retroactively rewarded or granted standard community grants as a token of appreciation.

## Supported Versions

During the Testnet phase, we solely provide security support for the `main`/`master` branch and the absolute latest released binaries. 

| Version | Supported          |
| ------- | ------------------ |
| `main` branch (Latest) | :white_check_mark: |
| Older Testnet Releases  | :x:                |

## Security Disclosures

When a security incident is resolved safely, we will publish a post-mortem to keep our community informed, adhering to responsible disclosure practices to protect the network until the patch is widely adopted.
