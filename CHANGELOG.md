# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `std::io::{Read, BufRead, Write, Seek}` and `embedded_io::{Read, BufRead, Write, Seek}` for
  `ReferenceMut`. They never fail on a borrow, so `try_borrow_mut` plus these impls gives
  embedded-io users a path without panics.
- `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` for `ReferenceMut`, matching `Reference`.
- `Clone` for `Reference`, without requiring `T: Clone`.
- `Serialize` (behind `serde`) and `defmt::Format` (behind `defmt`) for `Reference` and `ReferenceMut`.

### Changed

- The crate is now licensed under `EUPL-1.2 OR MPL-2.0`, at your option. Previously it was `EUPL-1.2` only.
- [**breaking**] `reference.clone()` on a `Reference` now returns another `Reference` instead of
  auto-dereferencing to `T::clone`. Use `T::clone(&reference)` or `(*reference).clone()` for the value.

### Internal

- Enabled `clippy::pedantic`.

## [0.2.0] - 2026-11-19

[0.2.0]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.2.0

### Added

- `OwnOrBorrow::Borrowed(&mut T)` variant and `From<&mut T>`.
- `try_borrow` and `try_borrow_mut` as non-panicking variants of `borrow` and `borrow_mut`.
- `is_owned`, `is_borrowed`, `into_owned` and `to_owned`.
- `Default` for `OwnOrBorrow`.
- `Debug` and `Display` for `Reference` and `ReferenceMut`.
- `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` for `Reference`.
- `std::io::{Read, Write, Seek}` for `OwnOrBorrow` behind the default `std` feature.
  A `RefCell` that is already borrowed yields an `ErrorKind::ResourceBusy` error.
- `embedded_io::{Read, Write, Seek}` (embedded-io 0.7) for `OwnOrBorrow` behind the
  `embedded-io` feature. These panic if the `RefCell` is already borrowed.
- `Serialize` and `Deserialize` for `OwnOrBorrow` behind the `serde` feature.
  Serializing fails if the `RefCell` is mutably borrowed; deserializing yields `Owned`.

### Changed

- [**breaking**] `borrow` and `borrow_mut` no longer borrow `self` for the lifetime `'a`.
  Previously, `borrow_mut` could only be called once on a value created from a `&RefCell`.
- [**breaking**] `try_into_owned` and the `RefCell` conversions return the unchanged
  `OwnOrBorrow` as the error, so the value is no longer lost on failure.
- The `RefCell` and `&RefCell` conversions are now `TryFrom` implementations, so
  `RefCell::try_from` works too. `try_into` keeps working through the standard blanket impl.
- [**breaking**] `try_into_owned` now succeeds for the owned `RefCell` variant.
- [**breaking**] Removed `TryIntoError`.
- [**breaking**] Removed the never-constructed `Reference::RefCellRef` and `ReferenceMut::RefCellRef` variants.
- [**breaking**] Bumped `defmt` to 1.2.
- `Debug`, `Display` and `defmt::Format` print `<borrowed>` instead of panicking when the
  `RefCell` is mutably borrowed.
- The crate is now always `#![no_std]`. The `std` feature links `std` only for the `std::io` impls.
- Updated to edition 2024. MSRV is now Rust 1.85.

## [0.1.1] - 2024-12-02

[0.1.1]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.1.1

### Internal

- Service release updating dependency patch versions.

## [0.1.0] - 2024-06-30

[0.1.0]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.1.0

### Added

- Initial release.
