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
#![forbid(unsafe_code)]
// Enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "std")]
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

impl<T> OwnOrBorrow<'_, T> {
    /// Initializes a new instance that owns data.
    pub const fn own(value: T) -> Self {
        Self::Owned(value)
    }

    /// Returns `true` if the value is owned, either directly or in an owned [`RefCell`].
    ///
    /// This checks the variant only, not whether a [`RefCell`] is currently borrowed.
    /// Use [`try_borrow`](Self::try_borrow) or [`try_borrow_mut`](Self::try_borrow_mut) for that.
    pub const fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_) | Self::RefCell(_))
    }

    /// Returns `true` if the value is a `&mut T` or a `&RefCell<T>`.
    ///
    /// This checks the variant only, not whether a [`RefCell`] is currently borrowed.
    /// An owned [`RefCell`] with an active borrow guard still returns `false`.
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
    ///
    /// # Errors
    /// Returns [`BorrowError`] if the value is behind a [`RefCell`] that is currently mutably borrowed.
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
    ///
    /// # Errors
    /// Returns [`BorrowMutError`] if the value is behind a [`RefCell`] that is currently borrowed.
    pub fn try_borrow_mut(&mut self) -> Result<ReferenceMut<'_, T>, BorrowMutError> {
        Ok(match self {
            Self::Owned(value) => ReferenceMut::Borrowed(value),
            Self::Borrowed(value) => ReferenceMut::Borrowed(value),
            Self::RefCell(cell) => cell.get_mut().into(),
            Self::RefCellRef(cell) => cell.try_borrow_mut()?.into(),
        })
    }

    /// Returns the owned value, or `self` if the value is borrowed.
    ///
    /// # Errors
    /// Returns the unchanged `self` for the `Borrowed` and `RefCellRef` variants.
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
        T::clone(&self.borrow())
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
        if let Ok(value) = self.try_borrow() {
            defmt::Format::format(&*value, fmt);
        } else {
            defmt::write!(fmt, "<borrowed>");
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

/// `std::io` impls for [`OwnOrBorrow`] and [`ReferenceMut`].
#[cfg(feature = "std")]
mod std_io {
    use super::{OwnOrBorrow, ReferenceMut};
    use std::io::{BufRead, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

    impl<T> OwnOrBorrow<'_, T> {
        /// Borrows the inner value mutably for one I/O call.
        ///
        /// A [`RefCell`](core::cell::RefCell) that is already borrowed becomes an
        /// [`ErrorKind::ResourceBusy`] error instead of a panic, since `std::io` can report it.
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

    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Read> Read for ReferenceMut<'_, T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            (**self).read(buf)
        }
    }

    /// Unlike on [`OwnOrBorrow`], this works because the guard keeps the borrow alive
    /// as long as the returned buffer.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: BufRead> BufRead for ReferenceMut<'_, T> {
        fn fill_buf(&mut self) -> Result<&[u8]> {
            (**self).fill_buf()
        }

        fn consume(&mut self, amount: usize) {
            (**self).consume(amount);
        }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Write> Write for ReferenceMut<'_, T> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            (**self).write(buf)
        }

        fn flush(&mut self) -> Result<()> {
            (**self).flush()
        }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: Seek> Seek for ReferenceMut<'_, T> {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
            (**self).seek(pos)
        }
    }
}

/// `embedded_io` impls for [`OwnOrBorrow`] and [`ReferenceMut`].
///
/// The [`OwnOrBorrow`] impls panic on a borrow conflict, because the inner error type cannot
/// express it. The [`ReferenceMut`] impls cannot conflict, since the guard holds the borrow.
#[cfg(feature = "embedded-io")]
mod embedded_io_impls {
    use super::{OwnOrBorrow, ReferenceMut};
    use embedded_io::{BufRead, ErrorType, Read, Seek, SeekFrom, Write};

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

    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: ErrorType> ErrorType for ReferenceMut<'_, T> {
        type Error = T::Error;
    }

    /// Never panics: the borrow is already held. Use
    /// [`OwnOrBorrow::try_borrow_mut`] to get a guard without panicking.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Read> Read for ReferenceMut<'_, T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            (**self).read(buf)
        }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: BufRead> BufRead for ReferenceMut<'_, T> {
        fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            (**self).fill_buf()
        }

        fn consume(&mut self, amount: usize) {
            (**self).consume(amount);
        }
    }

    /// Never panics: the borrow is already held. Use
    /// [`OwnOrBorrow::try_borrow_mut`] to get a guard without panicking.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Write> Write for ReferenceMut<'_, T> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            (**self).write(buf)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            (**self).flush()
        }
    }

    /// Never panics: the borrow is already held. Use
    /// [`OwnOrBorrow::try_borrow_mut`] to get a guard without panicking.
    #[cfg_attr(docsrs, doc(cfg(feature = "embedded-io")))]
    impl<T: Seek> Seek for ReferenceMut<'_, T> {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
            (**self).seek(pos)
        }
    }
}

/// Compiles and runs the README examples as doctests. They use `std::io`.
#[cfg(all(doctest, feature = "std"))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

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

/// Implements the traits that only need `&T` for a guard type, so `Reference`
/// and `ReferenceMut` behave the same as the value they point to.
macro_rules! impl_shared_traits {
    ($guard:ident) => {
        impl<T> core::borrow::Borrow<T> for $guard<'_, T> {
            #[inline]
            fn borrow(&self) -> &T {
                self
            }
        }

        impl<T> AsRef<T> for $guard<'_, T> {
            #[inline]
            fn as_ref(&self) -> &T {
                self
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $guard<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&**self, f)
            }
        }

        impl<T: fmt::Display> fmt::Display for $guard<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&**self, f)
            }
        }

        impl<T: PartialEq> PartialEq for $guard<'_, T> {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                **self == **other
            }
        }

        impl<T: Eq> Eq for $guard<'_, T> {}

        impl<T: PartialOrd> PartialOrd for $guard<'_, T> {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                (**self).partial_cmp(&**other)
            }
        }

        impl<T: Ord> Ord for $guard<'_, T> {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                (**self).cmp(&**other)
            }
        }

        impl<T: Hash> Hash for $guard<'_, T> {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                (**self).hash(state)
            }
        }

        #[cfg(feature = "defmt")]
        #[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
        impl<T: defmt::Format> defmt::Format for $guard<'_, T> {
            fn format(&self, fmt: defmt::Formatter) {
                defmt::Format::format(&**self, fmt)
            }
        }

        #[cfg(feature = "serde")]
        #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
        impl<T: serde::Serialize> serde::Serialize for $guard<'_, T> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                (**self).serialize(serializer)
            }
        }
    };
}

impl_shared_traits!(Reference);
impl_shared_traits!(ReferenceMut);

impl<T> core::borrow::BorrowMut<T> for ReferenceMut<'_, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> AsMut<T> for ReferenceMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

/// Copies the reference, or adds another shared [`RefCell`] borrow.
/// Does not need `T: Clone`.
impl<T> Clone for Reference<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Reference::Borrowed(value) => Reference::Borrowed(value),
            Reference::RefCell(guard) => Reference::RefCell(Ref::clone(guard)),
        }
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
