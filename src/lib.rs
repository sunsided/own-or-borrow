//! # Own or borrow your data.
//!
//! This crate provides the [`OwnOrBorrow`] type that wraps either owned data or a [`RefCell`]
//! borrowed reference to it. Think `Cow` for borrowing.
//!
//! ## `no_std` vs. `std`
//!
//! To use the crate in a `no_std` context, disable the `std` feature.
//!
//! ## Crate features
//! * `std` - Enables `std`; disabling enters `no_std` mode.
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
//! let mut value = OwnOrBorrow::from(refcell);
//!
//! assert_eq!(value.borrow().as_ref(), &42);
//! assert_eq!(value.borrow_mut().as_mut(), &mut 42);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
// Enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;

pub use crate::error::TryIntoError;
use core::cell::{Ref, RefCell, RefMut};
use core::ops::{Deref, DerefMut};

/// A type that provides either an owned value or [`RefCell`] borrowed reference to a value.
pub enum OwnOrBorrow<'a, T> {
    /// An owned value.
    Owned(T),
    /// A borrowed value.
    RefCell(RefCell<T>),
    /// A borrowed value.
    RefCellRef(&'a RefCell<T>),
}

/// A reference to borrowed or owned data.
pub enum Reference<'a, T> {
    /// An owned value.
    Borrowed(&'a T),
    /// A borrowed value.
    RefCell(Ref<'a, T>),
    /// A borrowed value.
    RefCellRef(&'a Ref<'a, T>),
}

/// A mutable reference to borrowed or owned data.
pub enum ReferenceMut<'a, T> {
    /// An owned value.
    Borrowed(&'a mut T),
    /// A borrowed value.
    RefCell(RefMut<'a, T>),
    /// A borrowed value.
    RefCellRef(&'a mut RefMut<'a, T>),
}

impl<'a, T> OwnOrBorrow<'a, T> {
    /// Initializes a new instance that owns data.
    pub fn own(value: T) -> Self {
        Self::Owned(value)
    }

    /// Borrows the inner value.
    pub fn borrow(&'a self) -> Reference<'a, T> {
        match self {
            OwnOrBorrow::Owned(value) => Reference::Borrowed(value),
            OwnOrBorrow::RefCell(ref_cell) => ref_cell.borrow().into(),
            OwnOrBorrow::RefCellRef(ref_cell) => ref_cell.borrow().into(),
        }
    }

    /// Borrows the inner value mutably.
    pub fn borrow_mut(&'a mut self) -> ReferenceMut<'a, T> {
        match self {
            OwnOrBorrow::Owned(value) => ReferenceMut::Borrowed(value),
            OwnOrBorrow::RefCell(ref_cell) => ref_cell.borrow_mut().into(),
            OwnOrBorrow::RefCellRef(ref_cell) => ref_cell.borrow_mut().into(),
        }
    }

    /// Implements [`TryInto`] behavior for owned variants.
    pub fn try_into_owned(self) -> Result<T, TryIntoError> {
        match self {
            OwnOrBorrow::Owned(value) => Ok(value),
            OwnOrBorrow::RefCell(_) => Err(TryIntoError::NotConvertible),
            OwnOrBorrow::RefCellRef(_) => Err(TryIntoError::NotConvertible),
        }
    }
}

impl<'a, T> From<RefCell<T>> for OwnOrBorrow<'a, T> {
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

impl<'a, T> Deref for Reference<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Reference::Borrowed(value) => value,
            Reference::RefCell(cell) => cell.deref(),
            Reference::RefCellRef(cell) => (*cell).deref(),
        }
    }
}

impl<'a, T> Deref for ReferenceMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReferenceMut::Borrowed(value) => value,
            ReferenceMut::RefCell(cell) => cell.deref(),
            ReferenceMut::RefCellRef(cell) => cell.deref(),
        }
    }
}

impl<'a, T> DerefMut for ReferenceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ReferenceMut::Borrowed(value) => value,
            ReferenceMut::RefCell(cell) => cell.deref_mut(),
            ReferenceMut::RefCellRef(cell) => cell.deref_mut(),
        }
    }
}

impl<'a, T> core::borrow::Borrow<T> for Reference<'a, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> core::borrow::Borrow<T> for ReferenceMut<'a, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> core::borrow::BorrowMut<T> for ReferenceMut<'a, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'a, T> AsRef<T> for Reference<'a, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> AsRef<T> for ReferenceMut<'a, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> AsMut<T> for ReferenceMut<'a, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
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

impl<'a, T> TryInto<RefCell<T>> for OwnOrBorrow<'a, T> {
    type Error = TryIntoError;

    fn try_into(self) -> Result<RefCell<T>, Self::Error> {
        match self {
            OwnOrBorrow::Owned(_) => Err(TryIntoError::NotConvertible),
            OwnOrBorrow::RefCell(cell) => Ok(cell),
            OwnOrBorrow::RefCellRef(_) => Err(TryIntoError::NotConvertible),
        }
    }
}

impl<'a, T> TryInto<&'a RefCell<T>> for OwnOrBorrow<'a, T> {
    type Error = TryIntoError;

    fn try_into(self) -> Result<&'a RefCell<T>, Self::Error> {
        match self {
            OwnOrBorrow::Owned(_) => Err(TryIntoError::NotConvertible),
            OwnOrBorrow::RefCell(_) => Err(TryIntoError::NotConvertible),
            OwnOrBorrow::RefCellRef(cell) => Ok(cell),
        }
    }
}

impl<'a, T> core::fmt::Debug for OwnOrBorrow<'a, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let data = self.borrow();
        let data = data.as_ref();
        core::fmt::Debug::fmt(data, f)
    }
}

impl<'a, T> core::fmt::Display for OwnOrBorrow<'a, T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let data = self.borrow();
        let data = data.as_ref();
        core::fmt::Display::fmt(data, f)
    }
}

#[cfg(feature = "defmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
impl<'a, T> defmt::Format for OwnOrBorrow<'a, T>
where
    T: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        let data = self.borrow();
        let data = data.as_ref();
        defmt::Format::format(data, fmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrow_owned_type() {
        let value = 42;
        let mut value = OwnOrBorrow::own(value);
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
    fn borrow_refcell() {
        let value = RefCell::new(42);
        let mut value = OwnOrBorrow::from(value);
        assert_eq!(value.borrow().as_ref(), &42);
        assert_eq!(value.borrow_mut().as_mut(), &mut 42);
    }

    #[test]
    fn debug() {
        let value = RefCell::new(42);
        let value = OwnOrBorrow::from(value);
        assert_eq!(format!("{:?}", value), "42");
    }

    #[test]
    #[cfg(feature = "std")]
    fn display() {
        let value = RefCell::new(42);
        let value = OwnOrBorrow::from(value);
        assert_eq!(format!("{}", value), "42");
    }
}
