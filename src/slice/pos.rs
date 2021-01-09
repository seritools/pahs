use crate::{Pos, Progress};

use super::NotEnoughDataError;

/// Convenience alias for byte slices.
pub type BytePos<'a> = SlicePos<'a, u8>;

/// A position in a slice.
///
/// A slice position tracks both the current input slice itself and the offset.
///
/// The offset is tracked separately from the slice to always know how far along the parsing is.
/// It is especially useful for error handling, as you can save the the offset
/// instead of the slice reference, allowing your error types to be `'static`.
#[derive(Debug)]
pub struct SlicePos<'a, T> {
    /// The offset to the beginning of the parsing process
    pub offset: usize,
    /// The current input slice
    pub s: &'a [T],
}

impl<'a, T> SlicePos<'a, T> {
    /// Creates a new slice position for the given slice, at offset `0`.
    #[inline]
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            offset: 0,
            s: slice,
        }
    }

    /// Advances the slice position by `offset` elements. Panics if the new position would
    /// be out of bounds.
    #[inline]
    pub fn advance_by(self, offset: usize) -> Self {
        Self {
            s: &self.s[offset..],
            offset: self.offset + offset,
        }
    }

    /// Convenience function to quickly convert the slice position
    /// into a failed [`Progress`](crate::Progress).
    #[inline]
    pub fn failure<U, E>(self, err: E) -> Progress<SlicePos<'a, T>, U, E> {
        Progress::failure(self, err)
    }

    /// Convenience function to quickly convert the slice position
    /// into a successful [`Progress`](crate::Progress).
    #[inline]
    pub fn success<R, E>(self, val: R) -> Progress<SlicePos<'a, T>, R, E> {
        Progress::success(self, val)
    }

    /// Takes `len` elements from the slice, advancing the slice position by that many elements.
    ///
    /// Fails if more elements are requested than there are left in the input slice.
    /// Also fails if zero elements are requested, in order to prevent infinite loops.
    #[inline]
    pub fn take(self, count: usize) -> Progress<SlicePos<'a, T>, &'a [T], NotEnoughDataError> {
        if count == 0 || count > self.s.len() {
            self.failure(NotEnoughDataError)
        } else {
            let matched = &self.s[0..count];
            self.advance_by(count).success(matched)
        }
    }
}

impl<'a, T> Pos for SlicePos<'a, T> {
    #[inline]
    fn zero() -> Self {
        SlicePos { offset: 0, s: &[] }
    }
}

impl<'a, T> Copy for SlicePos<'a, T> {}
impl<'a, T> Clone for SlicePos<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> PartialOrd for SlicePos<'a, T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for SlicePos<'a, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl<'a, T> PartialEq for SlicePos<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset.eq(&other.offset)
    }
}

impl<'a, T> Eq for SlicePos<'a, T> {}
