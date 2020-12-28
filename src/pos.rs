use crate::Progress;

/// A position in the parsed data
pub trait Pos: Ord + Copy {
    /// The initial position
    fn zero() -> Self;
}

impl Pos for usize {
    #[inline]
    fn zero() -> Self {
        0
    }
}

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
    pub fn fail<U>(self) -> Progress<SlicePos<'a, T>, U, ()> {
        Progress::failure(self, ())
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

    /// Converts the slice position into a successful [`Progress`](crate::Progress)
    /// if `val` is `Some`, with its wrapped value as the parse value, or into a failure otherwise.
    #[inline]
    pub fn success_opt<R>(self, val: Option<R>) -> Progress<SlicePos<'a, T>, R, ()> {
        if let Some(val) = val {
            self.success(val)
        } else {
            self.fail()
        }
    }

    /// Takes elements if `len` is `Some`, fails otherwise.
    ///
    /// See also [`take`](SlicePos::take).
    #[inline]
    pub fn take_opt(self, len: Option<usize>) -> Progress<SlicePos<'a, T>, &'a [T], ()> {
        if let Some(l) = len {
            self.take(l)
        } else {
            self.fail()
        }
    }

    /// Takes `len` elements from the slice, advancing the slice position by that many elements.
    ///
    /// Fails if more elements are requested than there are left in the input slice.
    /// Also fails if zero elements are requested, in order to prevent infinite loops.
    #[inline]
    pub fn take(self, len: usize) -> Progress<SlicePos<'a, T>, &'a [T], ()> {
        if len == 0 || len > self.s.len() {
            self.fail()
        } else {
            let matched = &self.s[0..len];
            self.advance_by(len).success(matched)
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
