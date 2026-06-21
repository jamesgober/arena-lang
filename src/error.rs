//! The error type returned by the fallible allocation path.

use core::fmt;

/// The reason a value could not be allocated into an [`Arena`](crate::Arena).
///
/// An arena addresses its slots with a 32-bit counter, so it can hold up to
/// `u32::MAX` values over its lifetime. Reaching that ceiling is the one
/// recoverable failure an allocation can hit; [`Arena::try_alloc`] reports it
/// through this type instead of aborting, so a caller building a very large tree
/// can stop cleanly rather than crash.
///
/// The enum is `#[non_exhaustive]`: should a later phase add a second backing with
/// its own failure mode, a `match` on this type must already account for it.
///
/// [`Arena::try_alloc`]: crate::Arena::try_alloc
///
/// # Examples
///
/// ```
/// use arena_lang::{Arena, ArenaError};
///
/// // The fallible path returns this type; the happy path yields a handle.
/// let mut arena: Arena<u32> = Arena::new();
/// let id = arena.try_alloc(7).expect("the first slot is always available");
/// assert_eq!(arena.get(id), Some(&7));
/// # let _ = ArenaError::CapacityExhausted;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArenaError {
    /// The arena's slot space is full: it already holds `u32::MAX` values and
    /// cannot represent another handle.
    ///
    /// This is unreachable for any realistic tree — it takes more than four
    /// billion live nodes — but it is reported rather than ignored so the limit
    /// is a defined boundary, never a silent wrap.
    CapacityExhausted,
}

impl fmt::Display for ArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CapacityExhausted => {
                f.write_str("arena is full: cannot allocate beyond u32::MAX values")
            }
        }
    }
}

impl core::error::Error for ArenaError {}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_capacity_exhausted_display_is_actionable() {
        let text = ArenaError::CapacityExhausted.to_string();
        assert!(text.contains("u32::MAX"), "{text}");
    }

    #[test]
    fn test_error_is_copy_and_equatable() {
        let a = ArenaError::CapacityExhausted;
        let b = a;
        assert_eq!(a, b);
    }
}
