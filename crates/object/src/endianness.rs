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
pub trait Number<'a, E>
where
    E: ParseError<Input<'a>>,
{
    /// Get endianness of the number.
    fn endianness() -> Endianness;

    /// Parse a `u8`.
    fn read_u8(input: Input<'a>) -> Result<u8, E>;

    /// Parse a `u16`.
    fn read_u16(input: Input<'a>) -> Result<u16, E>;

    /// Parse a `u32`.
    fn read_u32(input: Input<'a>) -> Result<u32, E>;

    /// Parse a `u64`.
    fn read_u64(input: Input<'a>) -> Result<u64, E>;
}

/// Type that implements [`Number`], which manipulates various little-endian
/// numbers.
pub struct LittleEndian;

impl<'a, E> Number<'a, E> for LittleEndian
where
    E: ParseError<Input<'a>>,
{
    fn endianness() -> Endianness {
        Endianness::Little
    }

    fn read_u8(input: Input<'a>) -> Result<u8, E> {
        le_u8(input)
    }

    fn read_u16(input: Input<'a>) -> Result<u16, E> {
        le_u16(input)
    }

    fn read_u32(input: Input<'a>) -> Result<u32, E> {
        le_u32(input)
    }

    fn read_u64(input: Input<'a>) -> Result<u64, E> {
        le_u64(input)
    }
}

/// Type that implements [`Number`], which manipulates various big-endian
/// numbers.
pub struct BigEndian;

impl<'a, E> Number<'a, E> for BigEndian
where
    E: ParseError<Input<'a>>,
{
    fn endianness() -> Endianness {
        Endianness::Big
    }

    fn read_u8(input: Input<'a>) -> Result<u8, E> {
        be_u8(input)
    }

    fn read_u16(input: Input<'a>) -> Result<u16, E> {
        be_u16(input)
    }

    fn read_u32(input: Input<'a>) -> Result<u32, E> {
        be_u32(input)
    }

    fn read_u64(input: Input<'a>) -> Result<u64, E> {
        be_u64(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_little_endian() {
        assert_eq!(<LittleEndian as Number<()>>::endianness(), Endianness::Little);
        assert_eq!(
            <LittleEndian as Number<()>>::read_u8(&42u8.to_le_bytes()),
            Ok((&[] as &[u8], 42))
        );
        assert_eq!(
            <LittleEndian as Number<()>>::read_u16(&42u16.to_le_bytes()),
            Ok((&[] as &[u8], 42))
        );
        assert_eq!(
            <LittleEndian as Number<()>>::read_u32(&42u32.to_le_bytes()),
            Ok((&[] as &[u8], 42))
        );
        assert_eq!(
            <LittleEndian as Number<()>>::read_u64(&42u64.to_le_bytes()),
            Ok((&[] as &[u8], 42))
        );
    }

    #[test]
    fn test_big_endian() {
        assert_eq!(<BigEndian as Number<()>>::endianness(), Endianness::Big);
        assert_eq!(<BigEndian as Number<()>>::read_u8(&42u8.to_be_bytes()), Ok((&[] as &[u8], 42)));
        assert_eq!(
            <BigEndian as Number<()>>::read_u16(&42u16.to_be_bytes()),
            Ok((&[] as &[u8], 42))
        );
        assert_eq!(
            <BigEndian as Number<()>>::read_u32(&42u32.to_be_bytes()),
            Ok((&[] as &[u8], 42))
        );
        assert_eq!(
            <BigEndian as Number<()>>::read_u64(&42u64.to_be_bytes()),
            Ok((&[] as &[u8], 42))
        );
    }
}
