//! Parsers for different number types.

use crate::Progress;

macro_rules! impl_number {
    ($num:ident) => {
        paste::paste! {
            #[doc = "Parses a `" $num "` in little-endian encoding."]
            #[inline]
            pub fn [<$num _le>]<'a, S>(
                _pd: &mut $crate::ParseDriver<S>,
                pos: $crate::slice::BytePos<'a>
            ) -> Progress<$crate::slice::BytePos<'a>, $num, $crate::slice::NotEnoughDataError> {
                pos
                    .take(::std::mem::size_of::<$num>())
                    .map(|n| {
                        // unwrap cannot fail since n.len() is always at least as big
                        // as the number type, because `consume` consumed at least
                        // that many bytes if we end up here
                        $num::from_le_bytes(::std::convert::TryInto::try_into(n).unwrap())
                    })
            }

            #[doc = "Parses a `" $num "` in big-endian encoding."]
            #[inline]
            pub fn [<$num _be>]<'a, S>(
                _pd: &mut $crate::ParseDriver<S>,
                pos: $crate::slice::BytePos<'a>
            ) -> Progress<$crate::slice::BytePos<'a>, $num, $crate::slice::NotEnoughDataError> {
                pos
                    .take(::std::mem::size_of::<$num>())
                    .map(|n| {
                        // unwrap cannot fail since n.len() is always at least as big
                        // as the number type, because `consume` consumed at least
                        // that many bytes if we end up here
                        $num::from_be_bytes(::std::convert::TryInto::try_into(n).unwrap())
                    })
            }
        }
    };

    ($ty:ident $($tys:ident)*) => {
        impl_number!($ty);
        impl_number!($($tys)*);
    };
}

impl_number!(
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
);

#[cfg(test)]
mod test {
    use crate::slice::{BytePos, NotEnoughDataError};
    use crate::ParseDriver;

    use super::*;

    #[test]
    fn fails_with_too_short_input() {
        let pd = &mut ParseDriver::new();

        let p = BytePos { offset: 0, s: &[] };

        let expected_u64 = Progress {
            pos: p,
            status: Err(NotEnoughDataError),
        };
        let expected_i8 = Progress {
            pos: p,
            status: Err(NotEnoughDataError),
        };

        assert_eq!(u64_le(pd, p), expected_u64);
        assert_eq!(u64_be(pd, p), expected_u64);
        assert_eq!(i8_le(pd, p), expected_i8);
        assert_eq!(i8_be(pd, p), expected_i8);
    }

    #[test]
    fn parses_ints_correctly() {
        let pd = &mut ParseDriver::new();

        let p = BytePos {
            offset: 0,
            s: &[0x01, 0x02, 0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D],
        };

        assert_eq!(
            u64_le(pd, p),
            Progress {
                pos: BytePos { offset: 8, s: &[] },
                status: Ok(0x0D_F0_0D_D0_04_03_02_01_u64),
            }
        );
        assert_eq!(
            i16_le(pd, p),
            Progress {
                pos: BytePos {
                    offset: 2,
                    s: &[0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D]
                },
                status: Ok(0x02_01_i16),
            }
        );

        assert_eq!(
            u64_be(pd, p),
            Progress {
                pos: BytePos { offset: 8, s: &[] },
                status: Ok(0x01_02_03_04_D0_0D_F0_0D_u64),
            }
        );
        assert_eq!(
            i16_be(pd, p),
            Progress {
                pos: BytePos {
                    offset: 2,
                    s: &[0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D]
                },
                status: Ok(0x01_02_i16),
            }
        );
    }
}
