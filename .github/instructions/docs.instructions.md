---
applyTo: "**/*.md"
---

# Documentation

- `README.md` is rendered on crates.io and docs.rs. Every link to a repo file and every
  image uses an absolute URL (`https://github.com/sunsided/own-or-borrow/...`). Only in-page
  anchors may be relative. `task lint:readme` checks this.
- `README.md` runs as doctests (`ReadmeDoctests` in `src/lib.rs`, with the `std` feature).
  Code blocks are tagged with their language. Untagged or `rust` blocks must compile. Use
  `rust,no_run` or `rust,ignore` only with a stated reason.
- The "Trait implementations" and "Not implemented" sections must match the impls in
  `src/lib.rs`. Flag a change that adds or removes a trait impl without updating them.
- Keep badges, the License section and the MSRV statement consistent with `Cargo.toml`.
