/// Helper trait for collections to push into.
pub trait Push<T> {
    /// The type that should be returning by [`finish`](Push::finish).
    type Output;

    /// Pushes a value into the collection.
    fn push(&mut self, value: T);

    /// Finishes pushing into the collection, returning the collection.
    fn finish(self) -> Self::Output;
}

impl<T> Push<T> for Vec<T> {
    type Output = Self;

    #[inline]
    fn push(&mut self, value: T) {
        self.push(value);
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self
    }
}

impl<T> Push<T> for &mut Vec<T> {
    type Output = Self;

    #[inline]
    fn push(&mut self, value: T) {
        (*self).push(value);
    }

    #[inline]
    fn finish(self) -> Self::Output {
        self
    }
}

impl<T> Push<T> for () {
    type Output = Self;

    /// Discards the value.
    #[inline]
    fn push(&mut self, _value: T) {}

    #[inline]
    fn finish(self) -> Self::Output {}
}
