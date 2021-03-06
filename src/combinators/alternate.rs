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
        let mut err_accumulator = self.err_accumulator;

        // accumulate the final progress
        let progress = err_accumulator.add_progress(self.current.unwrap());

        progress.map_err(|_| err_accumulator.finish())
    }
}

#[cfg(test)]
mod test {
    use crate::error_accumulator::AllErrorsAccumulator;
    use crate::slice::BytePos;
    use crate::{ParseDriver, Recoverable};

    #[derive(Debug, PartialEq)]
    pub struct TestError(bool);

    impl Recoverable for TestError {
        fn recoverable(&self) -> bool {
            self.0
        }
    }

    #[test]
    fn it_returns_the_first_successful_branch() {
        let input = &[0u8, 1, 2, 3, 4];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (res_pos, val) = pd
            .alternate(pos)
            .one(|_, pos| pos.failure(TestError(true)))
            .one(|_, pos| pos.advance_by(1).success(0u8))
            .one(|_, pos| pos.advance_by(2).success(1u8))
            .finish()
            .unwrap();

        assert_eq!(res_pos.offset, 1usize);
        assert_eq!(val, 0u8);
    }

    #[test]
    fn it_stops_at_irrecoverable_errors() {
        let input = &[0u8, 1, 2, 3, 4];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (res_pos, err) = pd
            .alternate(pos)
            .one(|_, pos| pos.failure(TestError(true)))
            .one(|_, pos| pos.failure(TestError(false)))
            .one(|_, pos| pos.advance_by(1).success(0u8))
            .finish()
            .unwrap_err();

        assert_eq!(res_pos.offset, 0usize);
        assert_eq!(err, TestError(false));
    }

    #[test]
    fn it_accumulates_all_errors() {
        let input = &[0u8, 1, 2, 3, 4];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (res_pos, err) = pd
            .alternate_accumulate_errors(pos, AllErrorsAccumulator::new())
            .one(|_, pos| pos.failure(TestError(true)))
            .one(|_, pos| pos.failure(TestError(true)))
            .one(|_, pos| pos.failure::<(), _>(TestError(false)))
            .one(|_, pos| pos.failure(TestError(true)))
            .finish()
            .unwrap_err();

        assert_eq!(res_pos.offset, 0usize);
        // last branch won't run because the third one was irrecoverable
        assert_eq!(err, &[TestError(true), TestError(true), TestError(false)]);
    }
}
