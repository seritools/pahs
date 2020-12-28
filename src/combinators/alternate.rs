use crate::error_accumulator::ErrorAccumulator;
use crate::{ParseDriver, Pos, Progress, Recoverable};

/// Try all parsers supplied via [`one`](crate::combinators::Alternate::one) in order
/// and return the value of the first one that successfully parses.
///
/// If none of the parsers were successful, returns the accumulated error.
#[must_use]
#[derive(Debug)]
pub struct Alternate<'pd, P: 'pd, T, E: 'pd, S, A: 'pd = ()> {
    driver: &'pd mut ParseDriver<S>,
    current: Option<Progress<P, T, E>>,
    pos: P,
    err_accumulator: A,
}

impl<'pd, P, T, E, S, A> Alternate<'pd, P, T, E, S, A>
where
    P: Pos,
    E: Recoverable,
    A: ErrorAccumulator<P, E>,
{
    fn run_one<F>(&mut self, parser: F)
    where
        F: FnOnce(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    {
        self.current = Some(parser(self.driver, self.pos))
    }

    /// Creates a new `Alternate` with the specified error accumulator.
    #[inline]
    pub fn new(driver: &'pd mut ParseDriver<S>, pos: P, err_accumulator: A) -> Self {
        Self {
            driver,
            current: None,
            pos,
            err_accumulator,
        }
    }

    /// Runs one parser if a previous one didn't parse successfully before.
    #[inline]
    pub fn one<F>(mut self, parser: F) -> Self
    where
        F: FnOnce(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    {
        match &mut self.current {
            None => self.run_one(parser),
            Some(Progress { status: Ok(..), .. }) => {
                // matched! skip all further parsers
            }
            Some(Progress { status: Err(e), .. }) if e.recoverable() => {
                // just matched on it, unwrap can't fail
                let current = self.current.take().unwrap();

                // accumulate it
                let _ = self.err_accumulator.add_progress(current);

                self.run_one(parser)
            }
            Some(Progress {
                status: Err(..), ..
            }) => {
                // irrecoverable, skip all further parsers
            }
        }

        self
    }

    /// Completes this `Alternate`, returning the progress of the first successful branch.
    ///
    /// If none of parsers were successful, it returns the accumulated errors
    ///
    /// Panics if no parser was run via [`one`](Alternate::one).
    #[inline]
    pub fn finish(self) -> Progress<P, T, A::Accumulated> {
        let accumulator = self.err_accumulator;
        self.current.unwrap().map_err(|_| accumulator.finish())
    }
}
