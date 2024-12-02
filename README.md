# Own or borrow your data.

[![Crates.io](https://img.shields.io/crates/v/own-or-borrow)](https://crates.io/crates/own-or-borrow)
[![Crates.io](https://img.shields.io/crates/l/own-or-borrow)](https://crates.io/crates/own-or-borrow)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sunsided/own-or-borrow/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/own_or_borrow)](https://docs.rs/own-or-borrow/)
[![codecov](https://codecov.io/gh/sunsided/own-or-borrow/graph/badge.svg?token=fYfqdDz6se)](https://codecov.io/gh/sunsided/own-or-borrow)

The `own-or-borrow` crate provides the `OwnOrBorrow` type, which can encapsulate either owned data or a `RefCell`-borrowed reference,
offering a flexible approach to data ownership and borrowing. This design is particularly useful in `no_std` environments,
as it allows for dynamic borrowing without relying on the standard library.

In contrast, Rust's standard `Cow` (Clone on Write) type is a smart pointer that enables efficient read-only access to
borrowed data and clones the data only when mutation is necessary. It operates with immutable references and requires the
`ToOwned` trait to clone the data when needed.

The key distinction between OwnOrBorrow and Cow lies in their borrowing mechanisms:

* `OwnOrBorrow`: Utilizes `RefCell` to allow for interior mutability, enabling mutable access to borrowed data at runtime.
  This is advantageous in scenarios where you need to mutate borrowed data without requiring ownership.
* `Cow`: Provides immutable access to borrowed data and clones it only when a mutation is required, ensuring that data is not cloned unnecessarily.

Therefore, `OwnOrBorrow` offers more flexibility in terms of mutability for borrowed data, which can be beneficial in certain
use cases, especially in `no_std` contexts. However, this flexibility comes with the overhead of runtime borrowing checks inherent to `RefCell`.

## Examples

You can create an `OwnOrBorrow` from an owned value:

```rust
use own_or_borrow::OwnOrBorrow;

fn example() {
    let mut value = OwnOrBorrow::own(42);

    assert_eq!(value.borrow().as_ref(), &42);
    assert_eq!(value.borrow_mut().as_mut(), &mut 42);
}
```

You can create an `OwnOrBorrow` from a `RefCell` and treat it the same way:

```rust
use own_or_borrow::OwnOrBorrow;
use core::cell::RefCell;

fn example() {
    let refcell = RefCell::new(42);
    let mut value = OwnOrBorrow::from(refcell);

    assert_eq!(value.borrow().as_ref(), &42);
    assert_eq!(value.borrow_mut().as_mut(), &mut 42);
}
```
