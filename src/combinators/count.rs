use crate::{ParseDriver, Pos, Progress, Recoverable};

/// Runs the specified parser `count` times, returning all parsed values in a `Vec`.
///
/// Don't need the parsed values? See [`skip_count`](skip_count).
#[inline]
pub fn count<P, T, E, F, S>(
    count: usize,
    mut parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, Vec<T>, E>
where
    P: Pos,
    E: Recoverable,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    move |pd, mut pos| {
        let mut vec = Vec::with_capacity(count);

        for _ in 0..count {
            match parser(pd, pos) {
                Progress {
                    status: Ok(val),
                    pos: new_pos,
                } => {
                    vec.push(val);
                    pos = new_pos;
                }

                Progress {
                    status: Err(err),
                    pos,
                } => return Progress::failure(pos, err),
            }
        }

        Progress::success(pos, vec)
    }
}

/// Runs the specified parser `count` times, discarding the parsed values.
#[inline]
pub fn skip_count<P, T, E, F, S>(
    count: usize,
    mut parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, (), E>
where
    P: Pos,
    E: Recoverable,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    move |pd, mut pos| {
        for _ in 0..count {
            match parser(pd, pos) {
                Progress {
                    status: Ok(_),
                    pos: new_pos,
                } => {
                    pos = new_pos;
                }

                Progress {
                    status: Err(err),
                    pos,
                } => return Progress::failure(pos, err),
            }
        }

        Progress::success(pos, ())
    }
}
