# Contributing to tableflow

Thanks for your interest in improving tableflow! This document explains the
workflow, the quality gates, and the rule that defines correctness here:
**conformance**.

By participating you agree to abide by our
[Code of Conduct](CODE_OF_CONDUCT.md).

## Branching model (git flow)

- **`main`** — released, tagged history. Protected; never pushed to directly.
- **`develop`** — integration branch for the next release. Protected.
- **feature branches** — branch off `develop`, named `feat/…`, `fix/…`,
  `docs/…`, etc.

Flow: `feature → PR → develop`, and for a release `develop → PR → main` followed
by a tag. Every change reaches `main` and `develop` through a pull request; CI
must be green before merging.

```bash
git switch develop && git pull
git switch -c feat/my-change
# …work…
git push -u origin feat/my-change     # then open a PR into develop
```

## Quality gates

Every PR must pass CI:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

Run them locally before pushing. Where the native toolchain is unavailable,
build and test inside Docker — see the [README](README.md#development).

## Conformance is the core rule

tableflow's correctness is defined by producing the **same export** as the
reference exporter, byte-for-byte after format-aware normalization (CSV rows,
XLSX cells, GeoJSON structure). A feature is not "done" until a golden fixture
proves it:

1. Add a form (content + submissions) to the corpus generator.
2. Regenerate the goldens (needs Docker and the reference exporter) — the
   generated export is written as a fixture **only if** tableflow already
   matches the reference after normalization.
3. Commit the fixture. The data-driven conformance test then enforces it with no
   external tooling needed.

**Never hand-write or hand-edit a golden export.** The reference exporter — not
us — defines the expected output.

## Style

Follow the surrounding code and the Rust API Guidelines; the project also tracks
the Microsoft "Pragmatic Rust Guidelines". Keep public items documented
(`missing_docs` is a warning), and prefer small, focused commits.

## Reporting bugs & proposing features

Open a GitHub issue with a minimal form + submissions that reproduce the problem
and the export you expected. For security issues, follow
[SECURITY.md](SECURITY.md) instead of filing a public issue.

## Licensing

Unless you state otherwise, contributions are dual-licensed under
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE), matching the project.
