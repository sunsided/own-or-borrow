---
applyTo: "Cargo.toml,release-plz.toml,.github/workflows/release-plz.yml,.github/workflows/publish-crates-io.yml,CHANGELOG.md"
---

# Crate and releases

- Versions are SemVer 2.0 (Cargo's interpretation). In `0.x`, the minor number is the
  breaking position. Pre-releases (`-rc.1`) are fine, but build metadata (`+...`) is not.
- Versions and `CHANGELOG.md` are managed by release-plz release PRs. Do not bump the version
  or edit generated changelog entries by hand in feature PRs.
- `Cargo.toml` keeps `description`, `license` (`EUPL-1.2 OR MPL-2.0`), `repository`,
  `readme`, `keywords` (at most 5) and `categories` (valid crates.io slugs), and declares
  `rust-version`.
- New dependencies must pass `cargo deny check`: allowed licenses, no advisories, no git or
  unknown-registry sources. They must not pull in `std` or `alloc` outside the `std` feature.
- `rust-version` is a signal, not a contract. Raising it is allowed when a dependency or a
  feature benefits. It needs a minor release and a changelog note, never a patch release.
- release-plz runs in git-only mode: it tags `vX.Y.Z` and creates GitHub Releases.
  `publish-crates-io.yml` publishes from the tag in the `crates-io` environment, via trusted
  publishing (`id-token: write`, no `CARGO_REGISTRY_TOKEN`). Keep it that way.
- Changelog groups and minor bumps come from imperative subject verbs (`release-plz.toml`).
  A subject that hides a feature behind "Update ..." lands under Changed. Flag it.
