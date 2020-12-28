use crate::{ParseDriver, Pos, Progress, Recoverable};

/// Wraps the specified `parser`, making it optional.
///
/// If `parser` was successful, the value is mapped to `Some(value)`.
/// Recoverable failures are mapped to successes, with `None` as value.
/// Irrecoverable failures stay that way.
#[inline]
pub fn optional<P, T, E, F, S>(
    pd: &mut ParseDriver<S>,
    pos: P,
    parser: F,
) -> Progress<P, Option<T>, E>
where
    P: Pos,
    E: Recoverable,
    F: FnOnce(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    let orig_pos = pos;

    match parser(pd, pos) {
        Progress {
            status: Ok(val),
            pos,
        } => Progress::success(pos, Some(val)),
        Progress {
            status: Err(e),
            pos,
        } => {
            if e.recoverable() {
                Progress::success(orig_pos, None)
            } else {
                Progress::failure(pos, e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestError(bool);

    impl Recoverable for TestError {
        fn recoverable(&self) -> bool {
            self.0
        }
    }

    #[test]
    fn successful_progress_gets_passed_through() {
        let mut pd = ParseDriver { state: () };
        let prog = optional(&mut pd, 0, |_, pos| {
            Progress::<_, _, TestError>::success(pos, "test")
        });

        // would panic if Progress::status isn't Ok
        assert_eq!(prog.unwrap(), (0usize, Some("test")));
    }

    #[test]
    fn recoverable_errors_turn_into_success_none() {
        let mut pd = ParseDriver { state: () };
        let prog = optional(&mut pd, 0, |_, pos| {
            Progress::<_, (), _>::failure(pos, TestError(true))
        });

        // would panic if Progress::status isn't Ok
        assert_eq!(prog.unwrap(), (0usize, None));
    }

    #[test]
    fn irrecoverable_errors_stay_failed() {
        let mut pd = ParseDriver { state: () };
        let prog = optional(&mut pd, 0, |_, pos| {
            Progress::<_, (), _>::failure(pos, TestError(false))
        });

        // would panic if Progress::status isn't Ok
        assert_eq!(prog.unwrap_err(), (0usize, TestError(false)));
    }
}
