# Security Policy

## Supported Versions

Noir is not fully audited and is not recommended for use in production.

| Version | Supported          |
| ------- | ------------------ |
| All versions | ❌ Not production ready |

## Reporting a Vulnerability

Noir sets out to be a secure language for developing zero-knowledge proofs. We thank you for taking the time to responsibly disclose any vulnerabilities you find.

### Where to Report

Bugs are unexpected behaviors in a system.

Vulnerabilities are bugs abusable for malicious intents (e.g. under-constrained bugs, miscompilations).

To report a vulnerability, [create a security advisory](https://github.com/noir-lang/noir/security/advisories/new) which privately discloses information to the Noir security team.

### Filling out the Reporting Form

You would be presented with a form when creating a security advisory:

- For _Affected products - Ecosystem_, choose _Other_ and input _Noir_
- For _Severity_, follow either the [_CVSS scoring_](https://docs.github.com/en/code-security/concepts/vulnerability-reporting-and-management/about-the-github-advisory-database#cvss-levels) or the simplified table below:

    | Severity | Impact: High | Impact: Medium | Impact: Low |
    | :--- | :--- | :--- | :--- |
    | **Likelihood: High** | Critical | High | Medium |
    | **Likelihood: Medium** | High | Medium | Low |
    | **Likelihood: Low** | Medium | Low | Low |

## Disclosure Process

1. Minimum 7 days before a new Noir version with ≥High security patches is released, a pre-announcement specifying i) CVE numbers of advisories that will be patched, and ii) the target release time, will be posted in [noir-security](https://groups.google.com/g/noir-security)
2. When the new Noir version is released, a separate announcement will be posted in the same channel
3. When the new Noir version is released, all advisories patched in the release will also be published on GitHub

### Receiving Security Updates

If you operate a Noir project in production, you are recommended to join [noir-security](https://groups.google.com/g/noir-security) with a Google account in order to receive email alerts of pre-announcements and plan for timely project patches at Noir releases.