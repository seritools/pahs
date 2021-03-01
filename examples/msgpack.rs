//! Super simple MessagePack pull parser, without validation

use std::convert::{TryFrom, TryInto};

use pahs::slice::num::{
    f32_be, f64_be, i16_be, i32_be, i64_be, i8_be, u16_be, u32_be, u64_be, u8_be,
};
use pahs::slice::SlicePos;
use pahs::{sequence, try_parse, ParseDriver};

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
        match MsgPack::parse(&mut pd, next_pos) {
            Progress {
                pos,
                status: Ok(elem),
            } => {
                next_pos = pos;
                println!("{:?}", elem);
            }
            Progress {
                pos,
                status: Err(Error::NoNextElement),
            } if pos.s.is_empty() => {
                break;
            }
            Progress {
                pos,
                status: Err(err),
            } => {
                eprintln!("{:?} at element at offset {:#X}", err, pos.offset);
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
    fn parse(pd: &mut Driver, start_pos: Pos<'a>) -> Progress<'a, Option<Self>, Error> {
        use MsgPack::*;
        let (pos, first_byte) = try_parse!(start_pos.take1().map_err(|_| Error::NoNextElement));

        let (pos, msgpack) = match first_byte {
            b if b >> 7 == 0 => (pos, UInt8(b & 0b_0111_1111)),
            b if b >> 4 == 0b1000 => (pos, Map(u32::from(b & 0b_0000_1111))),
            b if b >> 4 == 0b1001 => (pos, Array(u32::from(b & 0b_0000_1111))),
            b if b >> 5 == 0b101 => {
                try_parse!(pos
                    .take(usize::from(b & 0b1_1111))
                    .map_err(|_| Error::NotEnoughData)
                    .and_then(start_pos, |bytes| {
                        std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
                    })
                    .map(Str))
            }

            0xC0 => (pos, Nil),
            0xC1 => return Progress::failure(pos, Error::NeverUsedElement),
            0xC2 => (pos, False),
            0xC3 => (pos, True),

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
                let progress: Progress<'_, _, Error> = sequence!(
                    pd,
                    pos,
                    {
                        let n = parser;
                        let data = |_, pos: Pos<'a>| pos.take(n);
                    },
                    Bin(data)
                );
                try_parse!(progress)
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
                let progress: Progress<'_, _, Error> = sequence!(
                    pd,
                    pos,
                    {
                        let n = parser;
                        let (ext_type, data) = |pd, pos| Self::parse_ext_data(pd, pos, n);
                    },
                    Ext(ext_type, data)
                );
                try_parse!(progress)
            }

            0xCA => try_parse!(f32_be(pd, pos).map(Float32)),
            0xCB => try_parse!(f64_be(pd, pos).map(Float64)),

            0xCC => try_parse!(u8_be(pd, pos).map(UInt8)),
            0xCD => try_parse!(u16_be(pd, pos).map(UInt16)),
            0xCE => try_parse!(u32_be(pd, pos).map(UInt32)),
            0xCF => try_parse!(u64_be(pd, pos).map(UInt64)),

            0xD0 => try_parse!(i8_be(pd, pos).map(Int8)),
            0xD1 => try_parse!(i16_be(pd, pos).map(Int16)),
            0xD2 => try_parse!(i32_be(pd, pos).map(Int32)),
            0xD3 => try_parse!(i64_be(pd, pos).map(Int64)),

            0xD4 => try_parse!(Self::parse_ext_data(pd, pos, 1)
                .map(|(ext_type, d)| FixExt1(ext_type, d.try_into().unwrap()))),
            0xD5 => try_parse!(Self::parse_ext_data(pd, pos, 2)
                .map(|(ext_type, d)| FixExt2(ext_type, d.try_into().unwrap()))),
            0xD6 => try_parse!(Self::parse_ext_data(pd, pos, 4)
                .map(|(ext_type, d)| FixExt4(ext_type, d.try_into().unwrap()))),
            0xD7 => try_parse!(Self::parse_ext_data(pd, pos, 8)
                .map(|(ext_type, d)| FixExt8(ext_type, d.try_into().unwrap()))),
            0xD8 => try_parse!(Self::parse_ext_data(pd, pos, 16)
                .map(|(ext_type, d)| FixExt16(ext_type, d.try_into().unwrap()))),

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
                let progress: Progress<'_, _, Error> = sequence!(
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
                );
                try_parse!(progress)
            }

            0xDC => try_parse!(u16_be(pd, pos).map(|n| Array(u32::from(n)))),
            0xDD => try_parse!(u32_be(pd, pos).map(Array)),
            0xDE => try_parse!(u16_be(pd, pos).map(|n| Map(u32::from(n)))),
            0xDF => try_parse!(u32_be(pd, pos).map(Map)),
            b if b >> 5 == 0b111 => (pos, Int8(-((b & 0b1_1111) as i8))),
            _ => unreachable!(),
        };

        Progress::success(pos, Some(msgpack))
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
