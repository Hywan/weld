use nom::number::complete::{be_u16, be_u32, be_u64, be_u8, le_u16, le_u32, le_u64, le_u8};

use crate::{combinators::*, Input, Result};

/// Byte order of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    // Little endian byte order.
    Little,
    // Big endian byte order.
    Big,
}

/// Trait to read various numbers.
pub trait Number {
    /// Get endianness of the number.
    fn endianness() -> Endianness;

    /// Read a `u8`.
    fn read_u8<'a, E>(input: Input<'a>) -> Result<'a, u8, E>
    where
        E: ParseError<Input<'a>>;

    /// Read a `u16`.
    fn read_u16<'a, E>(input: Input<'a>) -> Result<'a, u16, E>
    where
        E: ParseError<Input<'a>>;

    /// Read a `u32`.
    fn read_u32<'a, E>(input: Input<'a>) -> Result<'a, u32, E>
    where
        E: ParseError<Input<'a>>;

    /// Read a `u64`.
    fn read_u64<'a, E>(input: Input<'a>) -> Result<'a, u64, E>
    where
        E: ParseError<Input<'a>>;

    /// Write a `u8`.
    fn write_u8(n: u8) -> [u8; 1];

    /// Write a `u16`.
    fn write_u16(n: u16) -> [u8; 2];

    /// Write a `u32`.
    fn write_u32(n: u32) -> [u8; 4];

    /// Write a `u64`.
    fn write_u64(n: u64) -> [u8; 8];
}

/// Type that implements [`Number`], which manipulates various little-endian
/// numbers.
pub struct LittleEndian;

impl Number for LittleEndian {
    fn endianness() -> Endianness {
        Endianness::Little
    }

    fn read_u8<'a, E>(input: Input<'a>) -> Result<'a, u8, E>
    where
        E: ParseError<Input<'a>>,
    {
        le_u8(input)
    }

    fn read_u16<'a, E>(input: Input<'a>) -> Result<'a, u16, E>
    where
        E: ParseError<Input<'a>>,
    {
        le_u16(input)
    }

    fn read_u32<'a, E>(input: Input<'a>) -> Result<'a, u32, E>
    where
        E: ParseError<Input<'a>>,
    {
        le_u32(input)
    }

    fn read_u64<'a, E>(input: Input<'a>) -> Result<'a, u64, E>
    where
        E: ParseError<Input<'a>>,
    {
        le_u64(input)
    }

    fn write_u8(n: u8) -> [u8; 1] {
        n.to_le_bytes()
    }

    fn write_u16(n: u16) -> [u8; 2] {
        n.to_le_bytes()
    }

    fn write_u32(n: u32) -> [u8; 4] {
        n.to_le_bytes()
    }

    fn write_u64(n: u64) -> [u8; 8] {
        n.to_le_bytes()
    }
}

/// Type that implements [`Number`], which manipulates various big-endian
/// numbers.
pub struct BigEndian;

impl Number for BigEndian {
    fn endianness() -> Endianness {
        Endianness::Big
    }

    fn read_u8<'a, E>(input: Input<'a>) -> Result<'a, u8, E>
    where
        E: ParseError<Input<'a>>,
    {
        be_u8(input)
    }

    fn read_u16<'a, E>(input: Input<'a>) -> Result<'a, u16, E>
    where
        E: ParseError<Input<'a>>,
    {
        be_u16(input)
    }

    fn read_u32<'a, E>(input: Input<'a>) -> Result<'a, u32, E>
    where
        E: ParseError<Input<'a>>,
    {
        be_u32(input)
    }

    fn read_u64<'a, E>(input: Input<'a>) -> Result<'a, u64, E>
    where
        E: ParseError<Input<'a>>,
    {
        be_u64(input)
    }

    fn write_u8(n: u8) -> [u8; 1] {
        n.to_be_bytes()
    }

    fn write_u16(n: u16) -> [u8; 2] {
        n.to_be_bytes()
    }

    fn write_u32(n: u32) -> [u8; 4] {
        n.to_be_bytes()
    }

    fn write_u64(n: u64) -> [u8; 8] {
        n.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_little_endian() {
        assert_eq!(LittleEndian::endianness(), Endianness::Little);
        assert_eq!(LittleEndian::read_u8::<()>(&42u8.to_le_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(LittleEndian::read_u16::<()>(&42u16.to_le_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(LittleEndian::read_u32::<()>(&42u32.to_le_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(LittleEndian::read_u64::<()>(&42u64.to_le_bytes()), Ok((&[] as &[u8], 42)));
    }

    #[test]
    fn test_big_endian() {
        assert_eq!(BigEndian::endianness(), Endianness::Big);
        assert_eq!(BigEndian::read_u8::<()>(&42u8.to_be_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(BigEndian::read_u16::<()>(&42u16.to_be_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(BigEndian::read_u32::<()>(&42u32.to_be_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(BigEndian::read_u64::<()>(&42u64.to_be_bytes()), Ok((&[] as &[u8], 42)));
    }
}
