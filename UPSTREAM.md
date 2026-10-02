# Upstream

This project is an independent repository derived from the MIT-licensed `Zamiell/hanabi-engine` project.

## Baseline

- Upstream repository: `https://github.com/Zamiell/hanabi-engine`
- Upstream default branch: `main`
- Audited/imported commit: `be00e7852571258af7847af35646c3571f620c64`
- Imported upstream tree: `2c1c2efb5d08032cd20021eb86fb2e78307789b2`
- Our baseline import commit: `e11f2520becc33207b200ff884fb35655d6ed187`
- Audit/import date: 2026-10-02

The imported tree SHA exactly matches the upstream commit tree SHA. The GitHub repository is intentionally independent rather than a GitHub fork. Because the available repository API cannot import foreign commit objects as ancestry, provenance is recorded explicitly by pinned commit/tree SHA instead of pretending that the repositories share Git ancestry.

## Sync policy

Treat Zamiell as an audited upstream source, not an automatic merge target.

For each upstream sync:

1. inspect upstream changes since the last reviewed revision;
2. record the new reviewed revision here;
3. port/cherry-pick equivalent changes only when compatible with Hanab Engine invariants;
4. run our validation and replay-parity checks before accepting the sync.

Do not blindly merge upstream `main` after the architecture diverges.

## Baseline validation status

The upstream Actions run for the pinned HEAD was cancelled by its execution-time limit, so it is not evidence of a green full suite.

Required local validation:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
scripts/check.sh
```

Record pre-existing failures before semantic changes.
