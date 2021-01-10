use crate::{ParseDriver, Pos, Progress, Push, Recoverable};

/// Runs the specified parser until it stops matching (but at least once),
/// collecting all values into a Vec.
///
/// Needs to run at least once to succeed.
///
/// See [`one_or_more_push_into`](one_or_more_push_into) if you want more control
/// over how the parsed values are collected.
#[inline]
pub fn one_or_more<P, T, E, F, S>(
    parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, Vec<T>, E>
where
    P: Pos,
    E: Recoverable,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    one_or_more_push_into(Vec::new, parser)
}

/// Runs the specified parser until it stops matching (but at least once),
/// collecting all values into the supplied [`Push`](Push) value.
///
/// Needs to run at least once to succeed.
#[inline]
pub fn one_or_more_push_into<P, T, E, Fp, S, C, Fc>(
    build_push: Fc,
    mut parser: Fp,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, C, E>
where
    P: Pos,
    E: Recoverable,
    Fp: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    C: Push<T>,
    Fc: FnOnce() -> C,
{
    move |pd, pos| {
        let mut coll = build_push();

        let (pos, val) = try_parse!(parser(pd, pos));
        coll.push(val);

        let mut curr_pos = pos;
        loop {
            match parser(pd, curr_pos) {
                Progress {
                    pos,
                    status: Ok(val),
                } => {
                    coll.push(val);
                    curr_pos = pos;
                }

                Progress {
                    pos,
                    status: Err(err),
                } if !err.recoverable() => return Progress::failure(pos, err),

                _err => return Progress::success(curr_pos, coll),
            }
        }
    }
}

/// Runs the specified parser until it stops matching,
/// collecting all values into a Vec.
///
/// See [`zero_or_more_push_into`](zero_or_more_push_into) if you want more control
/// over how the parsed values are collected.
#[inline]
pub fn zero_or_more<P, T, E, F, S>(
    parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, Vec<T>, E>
where
    P: Pos,
    E: Recoverable,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    zero_or_more_push_into(Vec::new, parser)
}

/// Runs the specified parser until it stops matching,
/// collecting all values into the supplied [`Push`](Push) value.
#[inline]
pub fn zero_or_more_push_into<P, T, E, Fp, S, C, Fc>(
    build_push: Fc,
    mut parser: Fp,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, C, E>
where
    P: Pos,
    E: Recoverable,
    Fp: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    C: Push<T>,
    Fc: FnOnce() -> C,
{
    move |pd, pos| {
        let mut coll = build_push();

        let mut curr_pos = pos;
        loop {
            match parser(pd, curr_pos) {
                Progress {
                    pos,
                    status: Ok(val),
                } => {
                    coll.push(val);
                    curr_pos = pos;
                }

                Progress {
                    pos,
                    status: Err(err),
                } if !err.recoverable() => return Progress::failure(pos, err),

                _err => return Progress::success(curr_pos, coll),
            }
        }
    }
}
