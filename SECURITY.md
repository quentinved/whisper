# Security Policy

Whisper handles secrets. Security reports are taken seriously and handled privately.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security problems.**

Report vulnerabilities through one of these private channels:

1. **GitHub Private Vulnerability Reporting** (preferred):
   https://github.com/quentinved/Whisper/security/advisories/new
2. **Email:** whisper@quentinvedrenne.com

Please include:
- A description of the issue and its impact
- Steps to reproduce (minimal proof-of-concept is helpful)
- Affected version(s) / commit SHA
- Your suggested fix, if you have one

You should get an initial response within **72 hours**. If you don't, please follow up — messages sometimes get missed.

## Disclosure process

1. We confirm the report and scope the impact.
2. We develop a fix and a regression test in a private branch.
3. We prepare a release with a CHANGELOG entry and a GitHub Security Advisory.
4. We credit the reporter in the advisory (unless you prefer to remain anonymous).
5. We publish the fix, the advisory, and the CVE if one applies.

For critical vulnerabilities, steps 3-5 happen on the same day as the fix.

## Scope

**In scope:**
- The `whisper-server` HTTP application (`applications/axum/server`)
- The `whisper-secrets` CLI (`applications/cli`)
- The encryption, repository, and Slack adapters
- The published npm packages (`whisper-secrets`, `@whisper-secrets/*`)
- The official hosted instance at `whisper.quentinvedrenne.com`

**Out of scope:**
- Self-hosted instances you deploy and configure yourself
- Social engineering attacks against maintainers or users
- Denial of service requiring more traffic than a reasonable free-tier instance handles
- Issues in dependencies that have no impact on Whisper (report upstream)
- Missing security headers without a demonstrated exploit path

## Supported versions

Only the latest minor release of each of `whisper-server` and `whisper-secrets` (CLI) receives security fixes. Older versions are not patched — please upgrade.

## Safe harbor

We won't pursue legal action against researchers who:
- Make a good-faith effort to avoid privacy violations, data loss, or service disruption
- Report the issue promptly through one of the channels above
- Don't publicly disclose before a fix is released

Thank you for helping keep Whisper and its users safe.
