use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

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

impl<K, V, S> Push<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn push(&mut self, (key, value): (K, V)) {
        self.insert(key, value);
    }
}

impl<K, V, S> Push<(K, V)> for &mut HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn push(&mut self, (key, value): (K, V)) {
        self.insert(key, value);
    }
}
