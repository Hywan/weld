pub use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    error::{ErrorKind, ParseError},
    sequence::tuple,
    Err,
};
use nom::{
    number::complete::{be_u16, be_u32, be_u64, be_u8, le_u16, le_u32, le_u64, le_u8},
    IResult, InputIter, ToUsize,
};

use crate::{Input, Result};

/// Like `take` but it “skips” the parsed value.
pub fn skip<'a, C, E>(count: C) -> impl Fn(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
where
    C: ToUsize,
    E: ParseError<Input<'a>>,
{
    let count = count.to_usize();

    move |input: Input| match input.slice_index(count) {
        Err(_needed) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof))),
        Ok(index) => Ok((&input[index..], &[])),
    }
}

/// Trait to parse various numbers.
pub trait NumberParser<'a, E>
where
    E: ParseError<Input<'a>>,
{
    /// Parse a `u8`.
    fn u8(input: Input<'a>) -> Result<u8, E>;

    /// Parse a `u16`.
    fn u16(input: Input<'a>) -> Result<u16, E>;

    /// Parse a `u32`.
    fn u32(input: Input<'a>) -> Result<u32, E>;

    /// Parse a `u64`.
    fn u64(input: Input<'a>) -> Result<u64, E>;
}

/// Type that implements [`NumberParser`], which parses various little-endian
/// numbers.
pub struct LittleEndian;

impl<'a, E> NumberParser<'a, E> for LittleEndian
where
    E: ParseError<Input<'a>>,
{
    fn u8(input: Input<'a>) -> Result<u8, E> {
        le_u8(input)
    }

    fn u16(input: Input<'a>) -> Result<u16, E> {
        le_u16(input)
    }

    fn u32(input: Input<'a>) -> Result<u32, E> {
        le_u32(input)
    }

    fn u64(input: Input<'a>) -> Result<u64, E> {
        le_u64(input)
    }
}

/// Type that implements [`NumberParser`], which parses various big-endian
/// numbers.
pub struct BigEndian;

impl<'a, E> NumberParser<'a, E> for BigEndian
where
    E: ParseError<Input<'a>>,
{
    fn u8(input: Input<'a>) -> Result<u8, E> {
        be_u8(input)
    }

    fn u16(input: Input<'a>) -> Result<u16, E> {
        be_u16(input)
    }

    fn u32(input: Input<'a>) -> Result<u32, E> {
        be_u32(input)
    }

    fn u64(input: Input<'a>) -> Result<u64, E> {
        be_u64(input)
    }
}
