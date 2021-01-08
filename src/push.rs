/// Helper trait for collections to push into.
pub trait Push<T> {
    /// Pushes a value into the collection.
    fn push(&mut self, value: T);
}

impl<T> Push<T> for Vec<T> {
    #[inline]
    fn push(&mut self, value: T) {
        self.push(value);
    }
}

impl<T> Push<T> for &mut Vec<T> {
    #[inline]
    fn push(&mut self, value: T) {
        (*self).push(value);
    }
}

impl<T> Push<T> for () {
    /// Discards the value.
    #[inline]
    fn push(&mut self, _value: T) {}
}
