//! Parser combinators.

mod alternate;
pub use alternate::*;

mod count;
pub use count::*;

mod optional;
pub use optional::*;

/// Runs parsers one after another, optionally saving their results.
/// Then, builds a value based on the saved results.
///
/// Short-circuits if any parser fails.
///
/// If you need access to the `ParseDriver` or the `Pos` when building the final value,
/// see [`sequence_with!`](crate::sequence_with) instead.
#[macro_export]
macro_rules! sequence {
    ($pd:expr, $pos:expr, {let $x:pat = $parser:expr; $($rest:tt)*}, $val:expr) => {
        $crate::sequence_with!($pd, $pos, {let $x = $parser; $($rest)*}, |_, _| $val)
    };

    ($pd:expr, $pos:expr, {$parser:expr; $($rest:tt)*}, $val:expr) => {
        $crate::sequence_with!($pd, $pos, {let _ = $parser; $($rest)*}, |_, _| $val)
    };

    ($pd:expr, $pos:expr, {}, $val:expr) => {
        $crate::sequence_with!($pd, $pos, {}, |_, _| $val)
    };
}

/// Runs parsers one after another, optionally saving their results.
/// Then, calls the creator function that builds a value based on the saved results.
///
/// Short-circuits if any parser fails.
#[macro_export]
macro_rules! sequence_with {
    ($pd:expr, $pos:expr, {let $x:pat = $parser:expr; $($rest:tt)*}, $creator:expr) => {
        match $parser(&mut *$pd, $pos) {
            $crate::Progress {
                status: ::std::result::Result::Ok($x),
                pos
            } => {
                $crate::sequence_with!($pd, pos, {$($rest)*}, $creator)
            },

            $crate::Progress {
                status: ::std::result::Result::Err(err),
                pos
            } => $crate::Progress {
                status: ::std::result::Result::Err(err.into()),
                pos
            }
        }
    };

    ($pd:expr, $pos:expr, {$parser:expr; $($rest:tt)*}, $creator:expr) => {
        $crate::sequence_with!($pd, $pos, {let _ = $parser; $($rest)*}, $creator)
    };

    ($pd:expr, $pos:expr, {}, $creator:expr) => {
        $crate::Progress::success($pos, $creator($pd, $pos))
    };
}
