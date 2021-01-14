//! Allows tracking multiple failures encountered in complex parsers or combinators.

use std::cmp::Ordering;

use crate::{Pos, Progress};

/// Allows tracking multiple failures encountered in complex parsers or combinators.
pub trait ErrorAccumulator<P, E> {
    /// As which type the accumulated errors will be returned (by [`finish`](Self::finish)).
    type Accumulated;

    /// Adds the specified error to the accumulation.
    fn add_err(&mut self, err: E, pos: P);

    /// Consumes and accumulates the error in the specified `Progress`, if any.
    #[inline]
    fn add_progress<T>(&mut self, progress: Progress<P, T, E>) -> Progress<P, T, ()>
    where
        P: Clone,
    {
        match progress {
            Progress {
                pos,
                status: Err(err),
            } => {
                self.add_err(err, pos.clone());
                Progress::failure(pos, ())
            }
            p @ Progress { .. } => p.map_err(|_| ()),
        }
    }

    /// Consumes the accumulator, returning the accumulated errors.
    fn finish(self) -> Self::Accumulated;
}

impl<P, E> ErrorAccumulator<P, E> for () {
    type Accumulated = ();

    #[inline(always)]
    fn add_err(&mut self, _err: E, _pos: P) {}

    #[inline(always)]
    fn finish(self) -> Self::Accumulated {}
}

/// Accumulator that only keeps the last added error.
///
/// Panics if no error was added.
#[derive(Debug)]
pub struct LastErrorOnly<E> {
    error: Option<E>,
}

impl<E> LastErrorOnly<E> {
    /// Creates a new accumulator.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E> Default for LastErrorOnly<E> {
    #[inline]
    fn default() -> Self {
        Self { error: None }
    }
}

impl<P, E> ErrorAccumulator<P, E> for LastErrorOnly<E> {
    type Accumulated = E;

    #[inline]
    fn add_err(&mut self, err: E, _pos: P) {
        self.error = Some(err);
    }

    /// Panics if no error was added.
    #[inline]
    fn finish(self) -> Self::Accumulated {
        self.error.unwrap()
    }
}

/// Accumulator that just stores all of the added errors
#[derive(Debug)]
pub struct AllErrorsAccumulator<E> {
    errors: Vec<E>,
}

impl<E> AllErrorsAccumulator<E> {
    /// Creates a new accumulator.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E> Default for AllErrorsAccumulator<E> {
    #[inline]
    fn default() -> Self {
        Self { errors: Vec::new() }
    }
}

impl<P, E> ErrorAccumulator<P, E> for AllErrorsAccumulator<E> {
    type Accumulated = Vec<E>;

    #[inline]
    fn add_err(&mut self, err: E, _pos: P) {
        self.errors.push(err);
    }

    #[inline]
    fn finish(self) -> Self::Accumulated {
        self.errors
    }
}

/// Accumulator that saves all "best" errors.
///
/// "Best" is defined as errors that happen at the furthest position into the input data.
/// If a "better" error is added, all previous errors are removed.
#[derive(Debug)]
pub struct AllBestErrorsAccumulator<P, E> {
    pos: P,
    errors: Vec<E>,
}

impl<P, E> AllBestErrorsAccumulator<P, E>
where
    P: Pos,
{
    /// Creates a new accumulator.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<P, E> Default for AllBestErrorsAccumulator<P, E>
where
    P: Pos,
{
    #[inline]
    fn default() -> Self {
        Self {
            pos: P::zero(),
            errors: Vec::new(),
        }
    }
}

impl<P, E> ErrorAccumulator<P, E> for AllBestErrorsAccumulator<P, E>
where
    P: Pos + Ord,
{
    type Accumulated = Vec<E>;

    #[inline]
    fn add_err(&mut self, err: E, pos: P) {
        match pos.cmp(&self.pos) {
            Ordering::Less => {
                // do nothing, our existing errors are better
            }
            Ordering::Greater => {
                // the new error is better, replace existing errors
                self.pos = pos;
                self.errors.clear();
                self.errors.push(err);
            }
            Ordering::Equal => {
                // multiple errors at the same point
                self.errors.push(err);
            }
        }
    }

    #[inline]
    fn finish(self) -> Self::Accumulated {
        self.errors
    }
}
