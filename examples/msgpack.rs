//! Super simple MessagePack pull parser, without validation

use std::convert::{TryFrom, TryInto};

use pahs::slice::num::{
    f32_be, f64_be, i16_be, i32_be, i64_be, i8_be, u16_be, u32_be, u64_be, u8_be,
};
use pahs::slice::SlicePos;
use pahs::{pahs, sequence, ParseDriver};

type Driver = ParseDriver<DriverState>;
type Pos<'a> = SlicePos<'a, u8>;
type Progress<'a, T, E> = pahs::Progress<Pos<'a>, T, E>;

#[derive(Debug, Default)]
struct DriverState {
    next: Option<()>,
}

fn main() {
    let file = std::env::args_os()
        .nth(1)
        .expect("usage: <executable> file");
    let msgpack_data = std::fs::read(file).expect("failed to read file");

    let mut pd = Driver::with_state(Default::default());
    let mut next_pos = Pos::new(&msgpack_data);

    loop {
        let (pos, result) = MsgPack::parse(&mut pd, next_pos).finish();
        match result {
            Ok(elem) => {
                next_pos = pos;
                println!("{:?}", elem);
            }
            Err(Error::NoNextElement) if pos.s.is_empty() => {
                break;
            }
            Err(e) => {
                eprintln!("{:?} at element at offset {:#X}", e, pos.offset);
                break;
            }
        }
    }
}

#[derive(Debug)]
enum MsgPack<'a> {
    Nil,
    False,
    True,
    Bin(&'a [u8]),
    Ext(u8, &'a [u8]),
    Float32(f32),
    Float64(f64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    FixExt1(u8, &'a [u8; 1]),
    FixExt2(u8, &'a [u8; 2]),
    FixExt4(u8, &'a [u8; 4]),
    FixExt8(u8, &'a [u8; 8]),
    FixExt16(u8, &'a [u8; 16]),
    Str(&'a str),
    Array(u32),
    Map(u32),
}

impl<'a> MsgPack<'a> {
    fn parse(pd: &mut Driver, start_pos: Pos<'a>) -> Progress<'a, Self, Error> {
        use MsgPack::*;
        let (pos, first_byte) = pahs!(start_pos.take1().map_err(|_| Error::NoNextElement));

        let progress: Progress<'_, _, Error> = match first_byte {
            b if b >> 7 == 0 => Ok((pos, UInt8(b & 0b_0111_1111))).into(),
            b if b >> 4 == 0b1000 => Ok((pos, Map(u32::from(b & 0b_0000_1111)))).into(),
            b if b >> 4 == 0b1001 => Ok((pos, Array(u32::from(b & 0b_0000_1111)))).into(),
            b if b >> 5 == 0b101 => pos
                .take(usize::from(b & 0b1_1111))
                .map_err(|_| Error::NotEnoughData)
                .and_then(start_pos, |bytes| {
                    std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
                })
                .map(Str),

            0xC0 => Ok((pos, Nil)).into(),
            0xC1 => return Err((pos, Error::NeverUsedElement)).into(),
            0xC2 => Ok((pos, False)).into(),
            0xC3 => Ok((pos, True)).into(),

            0xC4 | 0xC5 | 0xC6 => {
                let parser = match first_byte {
                    0xC4 => |pd, pos| u8_be(pd, pos).map(usize::from),
                    0xC5 => |pd, pos| u16_be(pd, pos).map(usize::from),
                    0xC6 => |pd, pos| {
                        u32_be(pd, pos)
                            .map(|n| usize::try_from(n).expect("u32 couldn't fit into usize"))
                    },
                    _ => unreachable!(),
                };
                sequence!(
                    pd,
                    pos,
                    {
                        let n = parser;
                        let data = |_, pos: Pos<'a>| pos.take(n);
                    },
                    Bin(data)
                )
            }

            0xC7 | 0xC8 | 0xC9 => {
                let parser = match first_byte {
                    0xC7 => |pd, pos| u8_be(pd, pos).map(usize::from),
                    0xC8 => |pd, pos| u16_be(pd, pos).map(usize::from),
                    0xC9 => |pd, pos| {
                        u32_be(pd, pos)
                            .map(|n| usize::try_from(n).expect("u32 couldn't fit into usize"))
                    },
                    _ => unreachable!(),
                };
                sequence!(
                    pd,
                    pos,
                    {
                        let n = parser;
                        let (ext_type, data) = |pd, pos| Self::parse_ext_data(pd, pos, n);
                    },
                    Ext(ext_type, data)
                )
            }

            0xCA => f32_be(pd, pos).map(Float32).to(),
            0xCB => f64_be(pd, pos).map(Float64).to(),

            0xCC => u8_be(pd, pos).map(UInt8).to(),
            0xCD => u16_be(pd, pos).map(UInt16).to(),
            0xCE => u32_be(pd, pos).map(UInt32).to(),
            0xCF => u64_be(pd, pos).map(UInt64).to(),

            0xD0 => i8_be(pd, pos).map(Int8).to(),
            0xD1 => i16_be(pd, pos).map(Int16).to(),
            0xD2 => i32_be(pd, pos).map(Int32).to(),
            0xD3 => i64_be(pd, pos).map(Int64).to(),

            0xD4 => Self::parse_ext_data(pd, pos, 1)
                .map(|(ext_type, d)| FixExt1(ext_type, d.try_into().unwrap())),
            0xD5 => Self::parse_ext_data(pd, pos, 2)
                .map(|(ext_type, d)| FixExt2(ext_type, d.try_into().unwrap())),
            0xD6 => Self::parse_ext_data(pd, pos, 4)
                .map(|(ext_type, d)| FixExt4(ext_type, d.try_into().unwrap())),
            0xD7 => Self::parse_ext_data(pd, pos, 8)
                .map(|(ext_type, d)| FixExt8(ext_type, d.try_into().unwrap())),
            0xD8 => Self::parse_ext_data(pd, pos, 16)
                .map(|(ext_type, d)| FixExt16(ext_type, d.try_into().unwrap())),

            0xD9 | 0xDA | 0xDB => {
                let parser = match first_byte {
                    0xD9 => |pd, pos| u8_be(pd, pos).map(usize::from),
                    0xDA => |pd, pos| u16_be(pd, pos).map(usize::from),
                    0xDB => |pd, pos| {
                        u32_be(pd, pos)
                            .map(|n| usize::try_from(n).expect("u32 couldn't fit into usize"))
                    },
                    _ => unreachable!(),
                };
                sequence!(
                    pd,
                    pos,
                    {
                        let n = parser;
                        let data = |_, pos: Pos<'a>| {
                            pos.take(n)
                                .map_err(|_| Error::NotEnoughData)
                                .and_then(start_pos, |bytes| {
                                    std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
                                })
                        };
                    },
                    Str(data)
                )
            }

            0xDC => u16_be(pd, pos).map(|n| Array(u32::from(n))).to(),
            0xDD => u32_be(pd, pos).map(Array).to(),
            0xDE => u16_be(pd, pos).map(|n| Map(u32::from(n))).to(),
            0xDF => u32_be(pd, pos).map(Map).to(),
            b if b >> 5 == 0b111 => Ok((pos, Int8(-((b & 0b1_1111) as i8)))).into(),
            _ => unreachable!(),
        };

        progress
    }

    fn parse_ext_data(
        pd: &mut Driver,
        pos: Pos<'a>,
        n: usize,
    ) -> Progress<'a, (u8, &'a [u8]), Error> {
        sequence!(
            pd,
            pos,
            {
                let ext_type = u8_be;
                let data = |_, pos: Pos<'a>| pos.take(n);
            },
            (ext_type, data)
        )
    }
}

#[derive(Debug)]
enum Error {
    NoNextElement,
    NotEnoughData,
    NeverUsedElement,
    InvalidUtf8,
}

impl From<pahs::slice::NotEnoughDataError> for Error {
    fn from(_: pahs::slice::NotEnoughDataError) -> Self {
        Self::NotEnoughData
    }
}
