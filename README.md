# Own or borrow your data.

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sunsided/own-or-borrow/rust.yml)
[![codecov](https://codecov.io/gh/sunsided/own-or-borrow/graph/badge.svg?token=fYfqdDz6se)](https://codecov.io/gh/sunsided/own-or-borrow)

This crate provides the `OwnOrBorrow` type that wraps either owned data or a `RefCell`
borrowed reference to it. Think `Cow` for borrowing.

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
