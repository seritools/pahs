//! Basic parsers for slices

use snafu::Snafu;

pub mod num;

use crate::{ParseDriver, Progress, SlicePos};

/// Matches the input slice against the `tag`, succeeding if both are equal.
#[inline]
pub fn tag<'a, T: PartialEq, S>(
    tag: &'a [T],
) -> impl Fn(&mut ParseDriver<S>, SlicePos<'a, T>) -> Progress<SlicePos<'a, T>, &'a [T], TagError> + 'a
{
    move |_, pos| {
        let (newpos, slice) = try_parse!(pos.take(tag.len()).map_err(|_| NotEnoughData.build()));

        if slice == tag {
            newpos.success(slice)
        } else {
            pos.failure(TagMismatch.build())
        }
    }
}

/// Errors that may happen when using [`tag`](tag).
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum TagError {
    /// The input slice was too short.
    NotEnoughData,
    /// The tag didn't match.
    TagMismatch,
}
