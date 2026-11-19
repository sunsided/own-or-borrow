//! Tests for the public API of `own_or_borrow`.

use core::cell::RefCell;
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};
use own_or_borrow::{OwnOrBorrow, Reference, ReferenceMut};

#[test]
fn borrow_owned_type() {
    let mut value = OwnOrBorrow::own(42);
    assert_eq!(value.borrow().as_ref(), &42);
    assert_eq!(value.borrow_mut().as_mut(), &mut 42);
}

#[test]
fn borrow_reference() {
    let value = &42;
    let mut value = OwnOrBorrow::own(value);
    assert_eq!(value.borrow().as_ref(), &&42);
    assert_eq!(value.borrow_mut().as_mut(), &&mut 42);
}

#[test]
fn borrow_mut_ref() {
    let mut target = 42;
    let mut value = OwnOrBorrow::from(&mut target);
    assert!(value.is_borrowed());
    *value.borrow_mut() += 1;
    assert_eq!(*value.borrow(), 43);
    assert_eq!(target, 43);
}

#[test]
fn borrow_refcell() {
    let mut value = OwnOrBorrow::from(RefCell::new(42));
    assert!(value.is_owned());
    assert_eq!(value.borrow().as_ref(), &42);
    assert_eq!(value.borrow_mut().as_mut(), &mut 42);
}

#[test]
fn deref_refcell() {
    let mut value = OwnOrBorrow::from(RefCell::new(42));
    assert_eq!(value.borrow().deref(), &42);
    assert_eq!(value.borrow_mut().deref_mut(), &mut 42);
}

/// Regression: `borrow_mut` used to borrow `self` for the whole lifetime `'a`.
#[test]
fn borrow_mut_refcell_ref_repeatedly() {
    let cell = RefCell::new(42);
    let mut value = OwnOrBorrow::from(&cell);
    *value.borrow_mut() += 1;
    *value.borrow_mut() += 1;
    assert_eq!(*value.borrow(), 44);
    assert_eq!(*cell.borrow(), 44);
}

#[test]
fn try_borrow_fails_while_mutably_borrowed() {
    let cell = RefCell::new(42);
    let mut value = OwnOrBorrow::from(&cell);
    let guard = cell.borrow_mut();
    assert!(value.try_borrow().is_err());
    assert!(value.try_borrow_mut().is_err());
    drop(guard);
    assert!(value.try_borrow_mut().is_ok());
}

#[test]
fn try_borrow_fails_on_owned_refcell_borrowed_through_variant() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    let OwnOrBorrow::RefCell(cell) = &value else {
        unreachable!()
    };
    let _guard = cell.borrow_mut();
    assert!(value.try_borrow().is_err());
    assert_eq!(format!("{value:?}"), "<borrowed>");
}

#[test]
fn debug() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    assert_eq!(format!("{value:?}"), "42");
}

#[test]
fn debug_does_not_panic_while_mutably_borrowed() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    let _guard = cell.borrow_mut();
    assert_eq!(format!("{value:?}"), "<borrowed>");
    assert_eq!(format!("{value}"), "<borrowed>");
}

#[test]
fn display() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    assert_eq!(format!("{value}"), "42");
    assert_eq!(format!("{}", value.borrow()), "42");
}

#[test]
fn default_is_owned() {
    let value: OwnOrBorrow<'_, i32> = OwnOrBorrow::default();
    assert!(value.is_owned());
    assert_eq!(value.try_into_owned().ok(), Some(0));
}

#[test]
fn try_into_owned() {
    let value = OwnOrBorrow::own(42);
    assert_eq!(value.try_into_owned().ok(), Some(42));
    let value = OwnOrBorrow::from(RefCell::new(42));
    assert_eq!(value.try_into_owned().ok(), Some(42));
}

#[test]
fn try_into_owned_from_refcell_ref_returns_input() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    let value = value.try_into_owned().expect_err("failed to fail");
    assert!(matches!(value, OwnOrBorrow::RefCellRef(_)));
}

#[test]
fn into_owned_clones_borrowed() {
    let cell = RefCell::new(42);
    assert_eq!(OwnOrBorrow::from(&cell).into_owned(), 42);
    let mut target = 7;
    assert_eq!(OwnOrBorrow::from(&mut target).into_owned(), 7);
}

#[test]
fn try_into_from_refcell() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    let _value: RefCell<i32> = value.try_into().expect("failed to unwrap refcell");
    let value = OwnOrBorrow::from(RefCell::new(42));
    let _value = RefCell::<i32>::try_from(value).expect("failed to unwrap refcell");
}

#[test]
fn try_into_from_refcell_ref() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    let _value: &RefCell<i32> = value.try_into().expect("failed to unwrap refcell ref");
}

#[test]
fn try_into_from_refcell_fails_for_refcell_ref() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    <OwnOrBorrow<'_, i32> as TryInto<RefCell<i32>>>::try_into(value).expect_err("failed to fail");
}

#[test]
fn try_into_from_refcell_ref_fails_for_refcell() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    <OwnOrBorrow<'_, i32> as TryInto<&RefCell<i32>>>::try_into(value).expect_err("failed to fail");
}

#[test]
fn reference_comparisons() {
    let cell = RefCell::new(2);
    let a = Reference::from(&1);
    let b = Reference::from(cell.borrow());
    assert!(a < b);
    assert_ne!(a, b);
    assert_eq!(a, Reference::from(&1));
    assert_eq!(a.cmp(&b), Ordering::Less);
}

#[test]
fn reference_hash_matches_value() {
    use core::hash::BuildHasher;
    let hasher = std::hash::RandomState::new();
    assert_eq!(hasher.hash_one(Reference::from(&42)), hasher.hash_one(42));
}

#[test]
fn try_borrow_all_variants() {
    let cell = RefCell::new(4);
    let mut target = 3;
    let mut values = [
        OwnOrBorrow::own(1),
        OwnOrBorrow::from(RefCell::new(2)),
        OwnOrBorrow::from(&mut target),
        OwnOrBorrow::from(&cell),
    ];
    for (expected, value) in (1..).zip(values.iter_mut()) {
        assert_eq!(*value.try_borrow().unwrap(), expected);
        *value.try_borrow_mut().unwrap() *= 10;
        assert_eq!(*value.borrow(), expected * 10);
    }
    assert_eq!(*cell.borrow(), 40);
}

#[test]
fn into_owned_owned_variants() {
    assert_eq!(OwnOrBorrow::own(1).into_owned(), 1);
    assert_eq!(OwnOrBorrow::from(RefCell::new(2)).into_owned(), 2);
}

#[test]
fn reference_traits() {
    use core::borrow::{Borrow, BorrowMut};
    let cell = RefCell::new(42);
    let reference = Reference::from(cell.borrow());
    assert_eq!(*Borrow::<i32>::borrow(&reference), 42);
    assert_eq!(format!("{reference:?}"), "42");
    drop(reference);

    let mut reference = ReferenceMut::from(cell.borrow_mut());
    *BorrowMut::<i32>::borrow_mut(&mut reference) += 1;
    assert_eq!(*Borrow::<i32>::borrow(&reference), 43);
    assert_eq!(reference.as_ref(), &43);
    assert_eq!(format!("{reference:?} {reference}"), "43 43");
    drop(reference);

    let mut value = 7;
    let reference = ReferenceMut::from(&mut value);
    assert_eq!(*reference, 7);
}

#[test]
fn reference_mut_comparisons() {
    use core::hash::BuildHasher;
    let cell = RefCell::new(2);
    let mut one = 1;
    let a = ReferenceMut::from(&mut one);
    let b = ReferenceMut::from(cell.borrow_mut());
    assert!(a < b);
    assert_ne!(a, b);
    assert_eq!(a.cmp(&b), Ordering::Less);
    let hasher = std::hash::RandomState::new();
    assert_eq!(hasher.hash_one(&b), hasher.hash_one(2));
}

#[test]
fn reference_clone_shares_refcell_borrow() {
    let cell = RefCell::new(42);
    let a = Reference::from(cell.borrow());
    let b = a.clone();
    drop(a);
    assert!(cell.try_borrow_mut().is_err(), "clone must keep the borrow");
    assert_eq!(*b, 42);
    drop(b);
    assert!(cell.try_borrow_mut().is_ok());

    let value = 7;
    let a = Reference::from(&value);
    assert_eq!(*a.clone(), 7);
}

#[test]
fn to_owned_clones_value_not_guard() {
    let value = OwnOrBorrow::from(RefCell::new(42));
    let owned: i32 = value.to_owned();
    assert_eq!(owned, 42);
}

#[cfg(feature = "serde")]
#[test]
fn serde_serializes_guards() {
    let cell = RefCell::new(42);
    assert_eq!(
        serde_json::to_string(&Reference::from(cell.borrow())).unwrap(),
        "42"
    );
    assert_eq!(
        serde_json::to_string(&ReferenceMut::from(cell.borrow_mut())).unwrap(),
        "42"
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    assert_eq!(serde_json::to_string(&value).unwrap(), "42");
    let value: OwnOrBorrow<'_, i32> = serde_json::from_str("7").unwrap();
    assert!(matches!(value, OwnOrBorrow::Owned(7)));
}

#[cfg(feature = "serde")]
#[test]
fn serde_fails_while_mutably_borrowed() {
    let cell = RefCell::new(42);
    let value = OwnOrBorrow::from(&cell);
    let _guard = cell.borrow_mut();
    assert!(serde_json::to_string(&value).is_err());
}

#[cfg(feature = "std")]
#[test]
fn std_io_forwards_and_reports_busy() {
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};
    let cell = RefCell::new(Cursor::new(std::vec::Vec::new()));
    let mut value = OwnOrBorrow::from(&cell);
    value.write_all(b"hello").unwrap();
    value.flush().unwrap();
    assert_eq!(value.seek(SeekFrom::Start(1)).unwrap(), 1);
    let mut buf = [0; 4];
    value.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"ello");

    let _guard = cell.borrow();
    let busy = std::io::ErrorKind::ResourceBusy;
    assert_eq!(value.write(b"!").unwrap_err().kind(), busy);
    assert_eq!(value.read(&mut buf).unwrap_err().kind(), busy);
    assert_eq!(value.flush().unwrap_err().kind(), busy);
    assert_eq!(value.seek(SeekFrom::Start(0)).unwrap_err().kind(), busy);
}

#[cfg(feature = "std")]
#[test]
fn std_io_on_guard() {
    use std::io::{BufRead, Cursor, Read, Seek, SeekFrom, Write};
    let cell = RefCell::new(Cursor::new(std::vec::Vec::new()));
    let mut value = OwnOrBorrow::from(&cell);
    let mut guard = value.try_borrow_mut().unwrap();
    guard.write_all(b"one\ntwo\n").unwrap();
    guard.flush().unwrap();
    assert_eq!(guard.seek(SeekFrom::Start(0)).unwrap(), 0);
    let mut line = std::string::String::new();
    guard.read_line(&mut line).unwrap();
    assert_eq!(line, "one\n");
    let mut rest = std::string::String::new();
    guard.read_to_string(&mut rest).unwrap();
    assert_eq!(rest, "two\n");
}

#[cfg(feature = "embedded-io")]
#[test]
fn embedded_io_forwards() {
    use embedded_io::{Read, Write};
    let mut out = [0u8; 5];
    let mut value = OwnOrBorrow::own(&mut out[..]);
    value.write_all(b"hello").unwrap();
    value.flush().unwrap();
    assert_eq!(&out, b"hello");

    let mut input: &[u8] = b"hi";
    let mut value = OwnOrBorrow::from(&mut input);
    let mut buf = [0u8; 2];
    value.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hi");
}

#[cfg(feature = "embedded-io")]
#[test]
fn embedded_io_seek_forwards() {
    use embedded_io::{ErrorKind, ErrorType, Seek, SeekFrom};
    struct Position(u64);
    impl ErrorType for Position {
        type Error = ErrorKind;
    }
    impl Seek for Position {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorKind> {
            match pos {
                SeekFrom::Start(n) => self.0 = n,
                _ => return Err(ErrorKind::Unsupported),
            }
            Ok(self.0)
        }
    }

    let cell = RefCell::new(Position(0));
    let mut value = OwnOrBorrow::from(&cell);
    assert_eq!(value.seek(SeekFrom::Start(3)), Ok(3));
    assert_eq!(cell.borrow().0, 3);
    assert_eq!(value.seek(SeekFrom::End(0)), Err(ErrorKind::Unsupported));
    assert_eq!(
        value.borrow_mut().seek(SeekFrom::End(0)),
        Err(ErrorKind::Unsupported)
    );
}

#[cfg(feature = "embedded-io")]
#[test]
fn embedded_io_on_guard() {
    use embedded_io::{BufRead, Read, Write};
    let mut out = [0u8; 2];
    {
        let cell = RefCell::new(&mut out[..]);
        let mut value = OwnOrBorrow::from(&cell);
        let mut guard = value.try_borrow_mut().unwrap();
        guard.write_all(b"ok").unwrap();
        guard.flush().unwrap();
    }
    assert_eq!(&out, b"ok");

    let mut input: &[u8] = b"abc";
    let mut value = OwnOrBorrow::from(&mut input);
    let mut guard = value.borrow_mut();
    assert_eq!(guard.fill_buf().unwrap(), b"abc");
    guard.consume(1);
    let mut buf = [0u8; 2];
    guard.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"bc");
}

#[cfg(feature = "embedded-io")]
#[test]
#[should_panic(expected = "already borrowed")]
fn embedded_io_panics_while_borrowed() {
    use embedded_io::Write;
    let mut out = [0u8; 1];
    let cell = RefCell::new(&mut out[..]);
    let mut value = OwnOrBorrow::from(&cell);
    let _guard = cell.borrow();
    let _ = value.write(b"!");
}
