---
applyTo: ".github/workflows/**,Taskfile.dist.yaml,.pre-commit-config.yaml,codecov.yml"
---

# CI, Taskfile and hooks stay in sync

- The `ci` task in `Taskfile.dist.yaml` mirrors the jobs in `.github/workflows/ci.yml`:
  `branch name`, `rustfmt`, `clippy`, `style`, `rustdoc`, `test`, `cargo-deny` and `MSRV`.
  `coverage` and `semver` have their own tasks but are not part of `task ci`. A change that adds, removes or renames a CI job, or
  changes its flags, must make the same change to the matching task, and the other way round.
- The feature sets are defined twice: `FEATURE_SETS` in the Taskfile and the `features`
  matrix of the `clippy` and `test` jobs. Keep them identical.
- Coverage leaves out `defmt` (`COVERAGE_FEATURES` in the Taskfile, the `coverage` job),
  because its `Format` impls cannot run on the host. Do not switch it back to `--all-features`.
- Local hooks in `.pre-commit-config.yaml` call Taskfile tasks. They must not duplicate
  commands.
- Every `uses:` is pinned to a full 40-character commit SHA, with a `# vX.Y.Z` comment. Reject
  tag or branch refs.
- The workflow sets `permissions: {}` at the top, and each job sets the least privilege it
  needs. Every `actions/checkout` sets `persist-credentials: false`.
- No `pull_request_target`. No `${{ github.event.* }}` or other untrusted expressions
  directly in `run:`. Pass them through `env:`.
- Do not rename `publish-crates-io.yml` or its `crates-io` environment. crates.io trusted
  publishing is bound to both. The `crates-io` job `needs:` the `verify` job.
- `scripts/check-branch.sh` is the single definition of allowed branch names. It is used by
  the Taskfile, the hook and the CI `branch name` job.
- New jobs that should gate merges must also be added to the `main` ruleset. Mention that in
  the PR.
