# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-11-19

[0.2.0]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.2.0

### Added

- `OwnOrBorrow::Borrowed(&mut T)` variant and `From<&mut T>`.
- `try_borrow` and `try_borrow_mut` as non-panicking variants of `borrow` and `borrow_mut`.
- `is_owned`, `is_borrowed`, `into_owned` and `to_owned`.
- `Default` for `OwnOrBorrow`.
- `Debug` and `Display` for `Reference` and `ReferenceMut`.
- `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` for `Reference`.

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
- [**breaking**] Bumped `defmt` to 1.x.
- `Debug`, `Display` and `defmt::Format` print `<borrowed>` instead of panicking when the
  `RefCell` is mutably borrowed.
- The crate is now always `no_std`. The `std` feature has no effect and is kept for compatibility.
- Updated to edition 2024. MSRV is now Rust 1.85.

## [0.1.1] - 2024-12-02

[0.1.1]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.1.1

### Internal

- Service release updating dependency patch versions.

## [0.1.0] - 2024-06-30

[0.1.0]: https://github.com/sunsided/own-or-borrow/releases/tag/v0.1.0

### Added

- Initial release.
