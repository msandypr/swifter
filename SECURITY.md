# Security Policy

## Supported Versions

Swifter receives active security updates on the following releases:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

---

## Reporting a Vulnerability

We take the security and integrity of **Swifter** seriously. Because Swifter handles executable parsing, micro-prefix symlinks, file execution, and download staging on macOS, security is paramount.

If you discover a security vulnerability, please do **NOT** open a public issue or discuss it publicly.

### How to Report
Please report security vulnerabilities privately using one of the following methods:
1. **GitHub Security Advisory (Preferred):** Open a private draft security advisory at [https://github.com/msandypr/swifter/security/advisories](https://github.com/msandypr/swifter/security/advisories).
2. **Direct Contact:** Email the maintainer directly at **`msandypr.dev@gmail.com`** with the subject line: `[SECURITY] Vulnerability in Swifter`.

### What to Include in Your Report
- A detailed description of the vulnerability.
- Clear reproduction steps or proof-of-concept (PoC).
- Impact assessment (e.g. arbitrary code execution, unauthorized file overwrites, privilege escalation).
- Any proposed remediation or patches if available.

### Response Timeline
- We will acknowledge receipt of your vulnerability report within **48 hours**.
- We will provide an assessment and status update within **7 business days**.
- If confirmed, a coordinated release and patch will be prepared before public disclosure.
