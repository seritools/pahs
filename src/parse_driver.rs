use crate::combinators::{optional, Alternate};
use crate::error_accumulator::{ErrorAccumulator, LastErrorOnly};
use crate::{Pos, Progress, Recoverable};

/// Maintains (optional) parsing state/context and serves as an easy entry point
/// for some of the combinators.
#[derive(Debug)]
pub struct ParseDriver<S = ()> {
    /// The parser state
    pub state: S,
}

impl ParseDriver<()> {
    /// Creates a new `ParseDriver` without state.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for ParseDriver<()> {
    #[inline]
    fn default() -> Self {
        Self { state: () }
    }
}

impl<S> ParseDriver<S> {
    /// Creates a new `ParseDriver` with `state` as initial state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        Self { state }
    }

    /// Wraps the specified `parser`, making it optional.
    ///
    /// If `parser` was successful, the value is mapped to `Some(value)`.
    /// Recoverable failures are mapped to successes, with `None` as value.
    /// Irrecoverable failures stay that way.
    #[inline]
    pub fn optional<P, T, E, F>(&mut self, pos: P, parser: F) -> Progress<P, Option<T>, E>
    where
        P: Pos,
        E: Recoverable,
        F: FnOnce(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    {
        optional(self, pos, parser)
    }

    /// Tries all parsers supplied via [`one`](crate::combinators::Alternate::one), in order,
    /// until one matches.
    ///
    /// If none of the parsers were successful, returns the error of the
    /// last run parser. If you want to retrieve the errors of the other parsers as well,
    /// see [`alternate_accumulate_errors`](ParseDriver::alternate_accumulate_errors).
    ///
    /// See [`Alternate`](crate::combinators::Alternate).
    #[inline]
    pub fn alternate<P, T, E>(&mut self, pos: P) -> Alternate<'_, P, T, E, S, LastErrorOnly<E>>
    where
        P: Pos,
        E: Recoverable,
    {
        Alternate::new(self, pos, LastErrorOnly::new())
    }

    /// Tries all parsers supplied via [`one`](crate::combinators::Alternate::one), in order,
    /// until one matches, accumulating errors of all failed parsers.
    ///
    /// If none of the parsers were successful, returns the error accumulated by
    /// the `error_accumulator`.
    ///
    /// See [`Alternate`](crate::combinators::Alternate).
    #[inline]
    pub fn alternate_accumulate_errors<P, T, E, A>(
        &mut self,
        pos: P,
        error_accumulator: A,
    ) -> Alternate<'_, P, T, E, S, A>
    where
        P: Pos,
        E: Recoverable,
        A: ErrorAccumulator<P, E>,
    {
        Alternate::new(self, pos, error_accumulator)
    }
}
