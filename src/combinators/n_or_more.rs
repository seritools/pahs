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
    move |pd, start_pos| {
        let mut coll = build_push();

        let (pos_after_first, val) = pahs!(parser(pd, start_pos));
        opt_assert!(pos_after_first != start_pos, "parser did not progress");
        coll.push(val);

        let mut curr_pos = pos_after_first;
        loop {
            match parser(pd, curr_pos) {
                Progress {
                    pos,
                    status: Ok(val),
                } => {
                    opt_assert!(curr_pos != pos, "parser did not progress");

                    coll.push(val);
                    curr_pos = pos;
                }

                Progress {
                    status: Err(err), ..
                } if !err.recoverable() => return Progress::failure(start_pos, err),

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
    move |pd, start_pos| {
        let mut coll = build_push();

        let mut curr_pos = start_pos;
        loop {
            match parser(pd, curr_pos) {
                Progress {
                    pos,
                    status: Ok(val),
                } => {
                    opt_assert!(curr_pos != pos, "parser did not progress");

                    coll.push(val);
                    curr_pos = pos;
                }

                Progress {
                    status: Err(err), ..
                } if !err.recoverable() => return Progress::failure(start_pos, err),

                _err => return Progress::success(curr_pos, coll),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::slice::num::u8_le;
    use crate::slice::BytePos;
    use crate::{ParseDriver, Progress, Recoverable};

    use super::{one_or_more, zero_or_more};

    #[derive(Debug, PartialEq)]
    enum Error {
        NotEnoughData,
        TooBig,
    }

    impl Recoverable for Error {
        fn recoverable(&self) -> bool {
            match self {
                Error::NotEnoughData => true,
                Error::TooBig => false,
            }
        }
    }

    fn under_64_parser<'a>(
        pd: &mut ParseDriver,
        pos: BytePos<'a>,
    ) -> Progress<BytePos<'a>, u8, Error> {
        u8_le(pd, pos)
            .map_err(|_| Error::NotEnoughData)
            .and_then(pos, |n| if n < 64 { Ok(n) } else { Err(Error::TooBig) })
    }

    #[test]
    fn one_or_more_works() {
        let input = &[0u8, 1, 2, 3, 4, 5, 6, 7, 8];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, vec) = one_or_more(under_64_parser)(pd, pos).unwrap();
        assert_eq!(new_pos.offset, 9);
        assert_eq!(vec, input);
    }

    #[test]
    fn one_or_more_errors_on_irrecoverable_and_rewinds_pos() {
        let input = &[0u8, 1, 2, 3, 64, 5];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, err) = one_or_more(under_64_parser)(pd, pos).unwrap_err();
        assert_eq!(new_pos.offset, 0);
        assert_eq!(err, Error::TooBig);
    }

    #[test]
    fn one_or_more_errors_on_empty() {
        let input = &[];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, err) = one_or_more(under_64_parser)(pd, pos).unwrap_err();
        assert_eq!(new_pos.offset, 0);
        assert_eq!(err, Error::NotEnoughData);
    }

    #[test]
    fn zero_or_more_works_for_valid_inputs() {
        let input = &[0u8, 1, 2, 3, 4, 5, 6, 7, 8];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, vec) = zero_or_more(under_64_parser)(pd, pos).unwrap();
        assert_eq!(new_pos.offset, 9);
        assert_eq!(vec, input);

        let input = &[];
        let pos = BytePos::new(input);

        let (new_pos, vec) = zero_or_more(under_64_parser)(pd, pos).unwrap();
        assert_eq!(new_pos.offset, 0);
        assert_eq!(vec, input);
    }

    #[test]
    fn zero_or_more_errors_on_irrecoverable_and_rewinds_pos() {
        let input = &[0u8, 1, 2, 3, 64, 5];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, err) = zero_or_more(under_64_parser)(pd, pos).unwrap_err();
        assert_eq!(new_pos.offset, 0);
        assert_eq!(err, Error::TooBig);
    }
}
