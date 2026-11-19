//! # Own or borrow your data.
//!
//! This crate provides the [`OwnOrBorrow`] type that wraps either owned data, a mutable
//! reference to it, or a [`RefCell`] borrowed reference to it. Think `Cow` for borrowing.
//!
//! ## `no_std`
//!
//! The crate does not require `alloc`. Disable the default `std` feature to use it in a
//! `no_std` context.
//!
//! ## Crate features
//! * `std` (default) - Implements the `std::io` traits `Read`, `Write` and `Seek`.
//! * `embedded-io` - Implements the [embedded-io](https://crates.io/crates/embedded-io) traits
//!   `Read`, `Write` and `Seek`.
//! * `serde` - Implements `Serialize` and `Deserialize` via [serde](https://crates.io/crates/serde).
//! * `defmt` - Enables deferred formatting support via the [defmt](https://crates.io/crates/defmt) crate.
//!
//! ## Examples
//!
//! You can create an [`OwnOrBorrow`] from an owned value:
//!
//! ```
//! use own_or_borrow::OwnOrBorrow;
//!
//! let mut value = OwnOrBorrow::own(42);
//!
//! assert_eq!(value.borrow().as_ref(), &42);
//! assert_eq!(value.borrow_mut().as_mut(), &mut 42);
//! ```
//!
//! You can create an [`OwnOrBorrow`] from a [`RefCell`] and treat it the same way:
//!
//! ```
//! use own_or_borrow::OwnOrBorrow;
//! use core::cell::RefCell;
//!
//! let refcell = RefCell::new(42);
//! let mut value = OwnOrBorrow::from(&refcell);
//!
//! *value.borrow_mut() += 1;
//! *value.borrow_mut() += 1;
//! assert_eq!(*refcell.borrow(), 44);
//! ```

#![no_std]
#![deny(unsafe_code)]
// Enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "std", test))]
extern crate std;

use core::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

/// A type that provides either an owned value, a mutable reference to a value,
/// or a [`RefCell`] borrowed reference to a value.
pub enum OwnOrBorrow<'a, T> {
    /// An owned value.
    Owned(T),
    /// An owned value behind a [`RefCell`].
    RefCell(RefCell<T>),
    /// A reference to a value behind a [`RefCell`].
    RefCellRef(&'a RefCell<T>),
    /// A mutable reference to a value.
    Borrowed(&'a mut T),
}

/// A reference to borrowed or owned data.
pub enum Reference<'a, T> {
    /// A plain reference.
    Borrowed(&'a T),
    /// A [`RefCell`] borrow guard.
    RefCell(Ref<'a, T>),
}

/// A mutable reference to borrowed or owned data.
pub enum ReferenceMut<'a, T> {
    /// A plain mutable reference.
    Borrowed(&'a mut T),
    /// A [`RefCell`] mutable borrow guard.
    RefCell(RefMut<'a, T>),
}

impl<'a, T> OwnOrBorrow<'a, T> {
    /// Initializes a new instance that owns data.
    pub const fn own(value: T) -> Self {
        Self::Owned(value)
    }

    /// Returns `true` if the value is owned, either directly or in an owned [`RefCell`].
    pub const fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_) | Self::RefCell(_))
    }

    /// Returns `true` if the value is borrowed.
    pub const fn is_borrowed(&self) -> bool {
        !self.is_owned()
    }

    /// Borrows the inner value.
    ///
    /// # Panics
    /// Panics if the value is behind a [`RefCell`] that is currently mutably borrowed.
    /// See [`try_borrow`](Self::try_borrow) for a non-panicking variant.
    pub fn borrow(&self) -> Reference<'_, T> {
        match self {
            Self::Owned(value) => Reference::Borrowed(value),
            Self::Borrowed(value) => Reference::Borrowed(value),
            Self::RefCell(cell) => cell.borrow().into(),
            Self::RefCellRef(cell) => cell.borrow().into(),
        }
    }

    /// Borrows the inner value, failing if it is behind a mutably borrowed [`RefCell`].
    pub fn try_borrow(&self) -> Result<Reference<'_, T>, BorrowError> {
        Ok(match self {
            Self::Owned(value) => Reference::Borrowed(value),
            Self::Borrowed(value) => Reference::Borrowed(value),
            Self::RefCell(cell) => cell.try_borrow()?.into(),
            Self::RefCellRef(cell) => cell.try_borrow()?.into(),
        })
    }

    /// Borrows the inner value mutably.
    ///
    /// # Panics
    /// Panics if the value is behind a [`RefCell`] that is currently borrowed.
    /// See [`try_borrow_mut`](Self::try_borrow_mut) for a non-panicking variant.
    pub fn borrow_mut(&mut self) -> ReferenceMut<'_, T> {
        match self {
            Self::Owned(value) => ReferenceMut::Borrowed(value),
            Self::Borrowed(value) => ReferenceMut::Borrowed(value),
            Self::RefCell(cell) => cell.get_mut().into(),
            Self::RefCellRef(cell) => cell.borrow_mut().into(),
        }
    }

    /// Borrows the inner value mutably, failing if it is behind a borrowed [`RefCell`].
    pub fn try_borrow_mut(&mut self) -> Result<ReferenceMut<'_, T>, BorrowMutError> {
        Ok(match self {
            Self::Owned(value) => ReferenceMut::Borrowed(value),
            Self::Borrowed(value) => ReferenceMut::Borrowed(value),
            Self::RefCell(cell) => cell.get_mut().into(),
            Self::RefCellRef(cell) => cell.try_borrow_mut()?.into(),
        })
    }

    /// Returns the owned value, or `self` if the value is borrowed.
    pub fn try_into_owned(self) -> Result<T, Self> {
        match self {
            Self::Owned(value) => Ok(value),
            Self::RefCell(cell) => Ok(cell.into_inner()),
            other => Err(other),
        }
    }

    /// Returns the owned value, cloning it if it is borrowed.
    ///
    /// # Panics
    /// Panics if the value is behind a [`RefCell`] that is currently mutably borrowed.
    pub fn into_owned(self) -> T
    where
        T: Clone,
    {
        match self.try_into_owned() {
            Ok(value) => value,
            Err(borrowed) => borrowed.to_owned(),
        }
    }

    /// Returns a clone of the value.
    ///
    /// # Panics
    /// Panics if the value is behind a [`RefCell`] that is currently mutably borrowed.
    pub fn to_owned(&self) -> T
    where
        T: Clone,
    {
        self.borrow().clone()
    }
}

impl<T: Default> Default for OwnOrBorrow<'_, T> {
    #[inline]
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

impl<'a, T> From<&'a mut T> for OwnOrBorrow<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Self::Borrowed(value)
    }
}

impl<T> From<RefCell<T>> for OwnOrBorrow<'_, T> {
    #[inline]
    fn from(value: RefCell<T>) -> Self {
        Self::RefCell(value)
    }
}

impl<'a, T> From<&'a RefCell<T>> for OwnOrBorrow<'a, T> {
    #[inline]
    fn from(value: &'a RefCell<T>) -> Self {
        Self::RefCellRef(value)
    }
}

impl<'a, T> TryFrom<OwnOrBorrow<'a, T>> for RefCell<T> {
    /// The unchanged input.
    type Error = OwnOrBorrow<'a, T>;

    fn try_from(value: OwnOrBorrow<'a, T>) -> Result<Self, Self::Error> {
        match value {
            OwnOrBorrow::RefCell(cell) => Ok(cell),
            other => Err(other),
        }
    }
}

impl<'a, T> TryFrom<OwnOrBorrow<'a, T>> for &'a RefCell<T> {
    /// The unchanged input.
    type Error = OwnOrBorrow<'a, T>;

    fn try_from(value: OwnOrBorrow<'a, T>) -> Result<Self, Self::Error> {
        match value {
            OwnOrBorrow::RefCellRef(cell) => Ok(cell),
            other => Err(other),
        }
    }
}

/// Prints the value, or `<borrowed>` if it is behind a mutably borrowed [`RefCell`].
impl<T: fmt::Debug> fmt::Debug for OwnOrBorrow<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_borrow() {
            Ok(value) => fmt::Debug::fmt(&*value, f),
            Err(_) => f.write_str("<borrowed>"),
        }
    }
}

/// Prints the value, or `<borrowed>` if it is behind a mutably borrowed [`RefCell`].
impl<T: fmt::Display> fmt::Display for OwnOrBorrow<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_borrow() {
            Ok(value) => fmt::Display::fmt(&*value, f),
            Err(_) => f.write_str("<borrowed>"),
        }
    }
}

#[cfg(feature = "defmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
impl<T: defmt::Format> defmt::Format for OwnOrBorrow<'_, T> {
    fn format(&self, fmt: defmt::Formatter) {
        match self.try_borrow() {
            Ok(value) => defmt::Format::format(&*value, fmt),
            Err(_) => defmt::write!(fmt, "<borrowed>"),
        }
    }
}

/// Serializes the value, failing if it is behind a mutably borrowed [`RefCell`].
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T: serde::Serialize> serde::Serialize for OwnOrBorrow<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.try_borrow()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

/// Deserializes into [`OwnOrBorrow::Owned`].
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for OwnOrBorrow<'_, T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::Owned)
    }
}

#[cfg(feature = "std")]
mod std_io {
    use super::{OwnOrBorrow, ReferenceMut};
    use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

    impl<T> OwnOrBorrow<'_, T> {
        fn io(&mut self) -> Result<ReferenceMut<'_, T>> {
            self.try_borrow_mut()
                .map_err(|e| Error::new(ErrorKind::ResourceBusy, e))
        }
    }

    /// Fails with [`ErrorKind::ResourceBusy`] if the value is behind a [`RefCell`](core::cell::RefCell)
    /// that is already borrowed.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Read> Read for OwnOrBorrow<'_, T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.io()?.read(buf)
        }
    }

    /// Fails with [`ErrorKind::ResourceBusy`] if the value is behind a [`RefCell`](core::cell::RefCell)
    /// that is already borrowed.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Write> Write for OwnOrBorrow<'_, T> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.io()?.write(buf)
        }

        fn flush(&mut self) -> Result<()> {
            self.io()?.flush()
        }
    }

    /// Fails with [`ErrorKind::ResourceBusy`] if the value is behind a [`RefCell`](core::cell::RefCell)
    /// that is already borrowed.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Seek> Seek for OwnOrBorrow<'_, T> {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
            self.io()?.seek(pos)
        }
    }
}

#[cfg(feature = "embedded-io")]
mod embedded_io_impls {
    use super::OwnOrBorrow;
    use embedded_io::{ErrorType, Read, Seek, SeekFrom, Write};

    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: ErrorType> ErrorType for OwnOrBorrow<'_, T> {
        type Error = T::Error;
    }

    /// # Panics
    /// Panics if the value is behind a [`RefCell`](core::cell::RefCell) that is already borrowed,
    /// because the inner error type cannot express that.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Read> Read for OwnOrBorrow<'_, T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.borrow_mut().read(buf)
        }
    }

    /// # Panics
    /// Panics if the value is behind a [`RefCell`](core::cell::RefCell) that is already borrowed,
    /// because the inner error type cannot express that.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Write> Write for OwnOrBorrow<'_, T> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.borrow_mut().flush()
        }
    }

    /// # Panics
    /// Panics if the value is behind a [`RefCell`](core::cell::RefCell) that is already borrowed,
    /// because the inner error type cannot express that.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Seek> Seek for OwnOrBorrow<'_, T> {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
            self.borrow_mut().seek(pos)
        }
    }
}

impl<T> Deref for Reference<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Reference::Borrowed(value) => value,
            Reference::RefCell(cell) => cell,
        }
    }
}

impl<T> Deref for ReferenceMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            ReferenceMut::Borrowed(value) => value,
            ReferenceMut::RefCell(cell) => cell,
        }
    }
}

impl<T> DerefMut for ReferenceMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        match self {
            ReferenceMut::Borrowed(value) => value,
            ReferenceMut::RefCell(cell) => cell,
        }
    }
}

impl<T> core::borrow::Borrow<T> for Reference<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> core::borrow::Borrow<T> for ReferenceMut<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> core::borrow::BorrowMut<T> for ReferenceMut<'_, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> AsRef<T> for Reference<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsRef<T> for ReferenceMut<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for ReferenceMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T> From<&'a T> for Reference<'a, T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Reference::Borrowed(value)
    }
}

impl<'a, T> From<&'a mut T> for ReferenceMut<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        ReferenceMut::Borrowed(value)
    }
}

impl<'a, T> From<Ref<'a, T>> for Reference<'a, T> {
    #[inline]
    fn from(value: Ref<'a, T>) -> Self {
        Reference::RefCell(value)
    }
}

impl<'a, T> From<RefMut<'a, T>> for ReferenceMut<'a, T> {
    #[inline]
    fn from(value: RefMut<'a, T>) -> Self {
        ReferenceMut::RefCell(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Reference<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for Reference<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for ReferenceMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for ReferenceMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: PartialEq> PartialEq for Reference<'_, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for Reference<'_, T> {}

impl<T: PartialOrd> PartialOrd for Reference<'_, T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: Ord> Ord for Reference<'_, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: Hash> Hash for Reference<'_, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;

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
    fn debug() {
        let value = OwnOrBorrow::from(RefCell::new(42));
        assert_eq!(format!("{:?}", value), "42");
    }

    #[test]
    fn debug_does_not_panic_while_mutably_borrowed() {
        let cell = RefCell::new(42);
        let value = OwnOrBorrow::from(&cell);
        let _guard = cell.borrow_mut();
        assert_eq!(format!("{:?}", value), "<borrowed>");
        assert_eq!(format!("{}", value), "<borrowed>");
    }

    #[test]
    fn display() {
        let value = OwnOrBorrow::from(RefCell::new(42));
        assert_eq!(format!("{}", value), "42");
        assert_eq!(format!("{}", value.borrow()), "42");
    }

    #[test]
    fn default_is_owned() {
        let value: OwnOrBorrow<'_, i32> = Default::default();
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
        <OwnOrBorrow<'_, i32> as TryInto<RefCell<i32>>>::try_into(value)
            .expect_err("failed to fail");
    }

    #[test]
    fn try_into_from_refcell_ref_fails_for_refcell() {
        let value = OwnOrBorrow::from(RefCell::new(42));
        <OwnOrBorrow<'_, i32> as TryInto<&RefCell<i32>>>::try_into(value)
            .expect_err("failed to fail");
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
        assert_eq!(format!("{:?}", reference), "42");
        drop(reference);

        let mut reference = ReferenceMut::from(cell.borrow_mut());
        *BorrowMut::<i32>::borrow_mut(&mut reference) += 1;
        assert_eq!(*Borrow::<i32>::borrow(&reference), 43);
        assert_eq!(reference.as_ref(), &43);
        assert_eq!(format!("{:?} {}", reference, reference), "43 43");
        drop(reference);

        let mut value = 7;
        let reference = ReferenceMut::from(&mut value);
        assert_eq!(*reference, 7);
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
        let err = value.write(b"!").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::ResourceBusy);
    }

    #[cfg(feature = "embedded-io")]
    #[test]
    fn embedded_io_forwards() {
        use embedded_io::{Read, Write};
        let mut out = [0u8; 5];
        let mut value = OwnOrBorrow::own(&mut out[..]);
        value.write_all(b"hello").unwrap();
        assert_eq!(&out, b"hello");

        let mut input: &[u8] = b"hi";
        let mut value = OwnOrBorrow::from(&mut input);
        let mut buf = [0u8; 2];
        value.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"hi");
    }
}
