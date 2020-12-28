#![warn(clippy::missing_inline_in_public_items)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

//! A parsing library.

/// An analog to `try!`/`?`, but for `Progress`
#[macro_export]
macro_rules! try_parse {
    ($e:expr) => {
        match $e {
            $crate::Progress {
                pos,
                status: ::std::result::Result::Ok(val),
            } => (pos, val),

            $crate::Progress {
                pos,
                status: ::std::result::Result::Err(val),
            } => {
                return $crate::Progress {
                    pos,
                    status: ::std::result::Result::Err(val.into()),
                }
            }
        }
    };
}

pub mod combinators;
pub mod error_accumulator;
mod parse_driver;
mod pos;
mod progress;
pub mod slice;

#[cfg(feature = "with_snafu")]
mod snafu;

pub use self::parse_driver::ParseDriver;
pub use self::pos::{BytePos, Pos, SlicePos};
pub use self::progress::Progress;

/// Indicates if an error allows a parent parser to recover and try something else.
///
/// Errors usually are irrecoverable if the input is well-formed,
/// but other constraints failed.
pub trait Recoverable {
    /// Returns `true` if the parse failure is recoverable, `false` otherwise.
    fn recoverable(&self) -> bool;
}
