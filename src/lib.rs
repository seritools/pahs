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

macro_rules! opt_assert {
    ($cond:expr $(,)?) => {
        if cfg!(any(test, feature = "loop_assert")) {
            assert!($cond);
        } else {
            debug_assert!($cond);
        }
    };

    ($cond:expr, $($arg:tt)+) => {{}
        if cfg!(any(test, feature = "loop_assert")) {
            assert!($cond, $($arg)+)
        } else {
            debug_assert!($cond, $($arg)+);
        }
    };
}

pub mod combinators;
pub mod error_accumulator;
mod parse_driver;
mod pos;
mod progress;
mod push;
pub mod slice;

#[cfg(feature = "with_snafu")]
mod snafu;

pub use self::parse_driver::ParseDriver;
pub use self::pos::Pos;
pub use self::progress::Progress;
pub use self::push::Push;

/// Indicates if an error allows a parent parser to recover and try something else.
///
/// Errors usually are irrecoverable if the input is well-formed,
/// but other constraints failed.
pub trait Recoverable {
    /// Returns `true` if the parse failure is recoverable, `false` otherwise.
    fn recoverable(&self) -> bool;
}
