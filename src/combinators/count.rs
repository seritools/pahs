use crate::{ParseDriver, Pos, Progress, Push};

/// Runs the specified parser `n` times, returning all parsed values in a `Vec`.
///
/// On failure, rewinds the position back to the initial position.
///
/// Note: This funtion pre-allocates the vector with the needed capacity for all `n` elements.
/// See [`count_push_into`](count_push_into) if you want more control over how the parsed
/// values are collected.
///
/// Don't need the parsed values at all? See [`skip_count`](skip_count).
#[inline]
pub fn count<P, T, E, F, S>(
    n: usize,
    parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, Vec<T>, E>
where
    P: Pos,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    count_push_into(n, move || Vec::with_capacity(n), parser)
}

/// Runs the specified parser `n` times, discarding the parsed values.
///
/// On failure, rewinds the position back to the initial position.
#[inline]
pub fn skip_count<P, T, E, F, S>(
    n: usize,
    parser: F,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, (), E>
where
    P: Pos,
    F: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
{
    count_push_into(n, || (), parser)
}

/// Runs the specified parser `n` times, pushing all values into the supplied [`Push`](Push)
/// value.
///
/// On failure, rewinds the position back to the initial position.
#[inline]
pub fn count_push_into<P, T, E, Fp, S, C, Fc>(
    n: usize,
    build_push: Fc,
    mut parser: Fp,
) -> impl FnOnce(&mut ParseDriver<S>, P) -> Progress<P, C::Output, E>
where
    P: Pos,
    Fp: FnMut(&mut ParseDriver<S>, P) -> Progress<P, T, E>,
    C: Push<T>,
    Fc: FnOnce() -> C,
{
    move |pd, mut pos| {
        let mut coll = build_push();
        let orig_pos = pos;

        for _ in 0..n {
            match parser(pd, pos) {
                Progress {
                    status: Ok(val),
                    pos: new_pos,
                } => {
                    coll.push(val);
                    pos = new_pos;
                }

                Progress {
                    status: Err(err), ..
                } => return Progress::failure(orig_pos, err),
            }
        }

        Progress::success(pos, coll.finish())
    }
}

#[cfg(test)]
mod test {
    use crate::slice::num::u8_le;
    use crate::slice::BytePos;
    use crate::ParseDriver;

    use super::{count, skip_count};

    #[test]
    fn it_works() {
        let input = &[0u8, 1, 2, 3, 4, 5, 6, 7, 8];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, vec) = count(6, u8_le)(pd, pos).unwrap();
        assert_eq!(new_pos.offset, 6);
        assert_eq!(new_pos.s, &input[6..]);
        assert_eq!(vec, &[0u8, 1, 2, 3, 4, 5]);

        let (new_pos, _) = skip_count(6, u8_le)(pd, pos).unwrap();
        assert_eq!(new_pos.offset, 6);
        assert_eq!(new_pos.s, &input[6..]);
    }

    #[test]
    fn it_rewinds_correctly_on_failure() {
        let input = &[0u8, 1, 2, 3, 4, 5, 6, 7, 8];
        let pos = BytePos::new(input);
        let pd = &mut ParseDriver::new();

        let (new_pos, _) = count(10, u8_le)(pd, pos).unwrap_err();
        assert_eq!(new_pos.offset, 0);

        let (new_pos, _) = skip_count(10, u8_le)(pd, pos).unwrap_err();
        assert_eq!(new_pos.offset, 0);
    }
}
