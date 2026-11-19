# Own or borrow your data.

[![Crates.io](https://img.shields.io/crates/v/own-or-borrow)](https://crates.io/crates/own-or-borrow)
[![Crates.io](https://img.shields.io/crates/l/own-or-borrow)](https://crates.io/crates/own-or-borrow)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sunsided/own-or-borrow/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/own_or_borrow)](https://docs.rs/own-or-borrow/)
[![codecov](https://codecov.io/gh/sunsided/own-or-borrow/graph/badge.svg?token=fYfqdDz6se)](https://codecov.io/gh/sunsided/own-or-borrow)

The `own-or-borrow` crate provides the `OwnOrBorrow` type. It holds an owned value, a `&mut T`, an owned
`RefCell<T>` or a `&RefCell<T>`, and gives the same read and write access to all four. It works in `no_std`
without `alloc`.

## Compared to `Cow` and `mayber`

| | `Cow<'a, B>` | [`mayber::Maybe<R, T>`](https://crates.io/crates/mayber) | `OwnOrBorrow<'a, T>` |
|---|---|---|---|
| Holds | `&B` or `B::Owned` | any `R` or `T` | `T`, `&mut T`, `RefCell<T>` or `&RefCell<T>` |
| Mutable access to borrowed data | no, clones on write | with `MaybeMut` (`&mut T`) | yes, including shared `RefCell`s |
| Requires `ToOwned` | yes | no | no |
| `RefCell` support | no | no | yes, with `try_borrow` and `try_borrow_mut` |
| `no_std` without `alloc` | no | yes | yes |

The price of `RefCell` support is the runtime borrow check that comes with it. `borrow` and `borrow_mut`
panic on a conflicting `RefCell` borrow; `try_borrow` and `try_borrow_mut` return an error instead.

## Examples

You can create an `OwnOrBorrow` from an owned value:

```rust
use own_or_borrow::OwnOrBorrow;

let mut value = OwnOrBorrow::own(42);

assert_eq!(value.borrow().as_ref(), &42);
assert_eq!(value.borrow_mut().as_mut(), &mut 42);
```

You can create an `OwnOrBorrow` from a `RefCell` and treat it the same way:

```rust
use own_or_borrow::OwnOrBorrow;
use core::cell::RefCell;

let refcell = RefCell::new(42);
let mut value = OwnOrBorrow::from(&refcell);

*value.borrow_mut() += 1;
assert_eq!(*refcell.borrow(), 43);
```

## Trait implementations

* `OwnOrBorrow<T>`: `From<&mut T>`, `From<RefCell<T>>`, `From<&RefCell<T>>`, `Default`, `Debug`,
  `Display`, `std::io` and `embedded_io` `Read`, `Write` and `Seek`, `Serialize`, `Deserialize`,
  `defmt::Format`. `RefCell<T>` and `&RefCell<T>` implement `TryFrom<OwnOrBorrow<T>>`.
* `Reference<T>` (from `borrow`): `Deref`, `AsRef`, `Borrow`, `Clone`, `From<&T>`, `From<Ref<T>>`,
  `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Debug`, `Display`, `Serialize`, `defmt::Format`.
  `Clone` needs no `T: Clone`: it copies the reference or adds another shared `RefCell` borrow.
* `ReferenceMut<T>` (from `borrow_mut`): everything `Reference` has except `Clone` and `From<&T>`,
  plus `DerefMut`, `AsMut`, `BorrowMut`, `From<&mut T>`, `From<RefMut<T>>`, and `std::io` and
  `embedded_io` `Read`, `BufRead`, `Write` and `Seek`.

Comparison and hashing use the inner value, so a `Reference` hashes like the `T` it points to.

The I/O traits on `ReferenceMut` never fail on a borrow, because the guard already holds it. With
`embedded-io`, take the guard with `try_borrow_mut` to avoid the panic of the `OwnOrBorrow` impls:

```rust
use core::cell::RefCell;
use own_or_borrow::OwnOrBorrow;

let cell = RefCell::new(Vec::new());
let mut value = OwnOrBorrow::from(&cell);

if let Ok(mut guard) = value.try_borrow_mut() {
    std::io::Write::write_all(&mut guard, b"hello").unwrap();
}
assert_eq!(cell.borrow().as_slice(), b"hello");
```

### Not implemented

* `From<T>`: it would overlap with `From<&mut T>` and `From<RefCell<T>>`. Use `OwnOrBorrow::own`.
* `Clone` and `Copy` for `OwnOrBorrow`: a `&mut T` cannot be cloned, and cloning into an owned
  value would silently cut the link to the original.
* `PartialEq`, `Ord` and `Hash` for `OwnOrBorrow`: comparing would panic on a mutably borrowed
  `RefCell`, and a key whose `RefCell` changes breaks a `HashMap`. Compare `*a.borrow() == *b.borrow()`.
* `BufRead` for `OwnOrBorrow`: `fill_buf` returns a slice that cannot outlive the temporary
  `RefCell` guard. Use `borrow_mut` and call it on the `ReferenceMut`.

## Crate features

* `std` (default): the `std::io` traits. On `OwnOrBorrow`, a `RefCell` that is already
  borrowed yields an `ErrorKind::ResourceBusy` error.
* `embedded-io`: the `embedded_io` traits (embedded-io 0.7). On `OwnOrBorrow`, these panic if
  the `RefCell` is already borrowed, because the inner error type cannot express that.
* `serde`: `Serialize` and `Deserialize`. Serializing an `OwnOrBorrow` fails if the `RefCell` is
  mutably borrowed; deserializing always yields an owned value.
* `defmt`: `defmt::Format`.

Disable default features for `no_std`. The crate needs no `alloc` and requires Rust 1.85 or later.
