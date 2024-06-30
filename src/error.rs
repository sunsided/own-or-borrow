use core::fmt::{Display, Formatter};

/// Errors from [`TryInto`] traits and related.
#[derive(Debug)]
pub enum TryIntoError {
    /// The type is not convertible.
    NotConvertible,
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for TryIntoError {}

impl Display for TryIntoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TryIntoError::NotConvertible => f.write_str("Invalid Type"),
        }
    }
}
