<!--
Thanks for sending a PR! Please fill in the sections below.
For non-trivial changes, link an issue you've discussed first.
-->

## Summary

<!-- 1-3 sentences: what changed and why. -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor / internal change
- [ ] Documentation
- [ ] CI / build / release tooling
- [ ] Other:

## Related issues

<!-- e.g. "Closes #123", "Related to #456". Delete if none. -->

## Test plan

<!-- How did you verify this works? -->

- [ ] `cargo fmt` clean
- [ ] `cargo clippy --workspace --all-targets --features test-utils -- -D warnings` clean
- [ ] `cargo test --workspace` passes locally
- [ ] Added or updated tests for the change
- [ ] Manually verified the affected flow (describe below)

<!-- Manual verification steps, if any: -->

## Documentation / changelog

- [ ] Updated `README.md` / `CLAUDE.md` if user-facing or developer-facing behavior changed
- [ ] Added a `CHANGELOG.md` entry under a new dated section (if user-visible)

## Checklist

- [ ] I read [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] This change does not introduce a security regression — if you are unsure, please flag it in the summary
