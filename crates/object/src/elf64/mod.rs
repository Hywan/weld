//! Elf64 support.

use std::{fmt, io, num::NonZeroU64, ops::Add};

use nom::Err::Error;

use crate::{combinators::*, Input, Number, Read, Result, Write};

mod data;
mod file;
mod program;
mod section;
mod symbol;

pub use data::*;
pub use file::*;
pub use program::*;
pub use section::*;
pub use symbol::*;

/// An address within the file.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Address(pub u64);

impl Read<u64> for Address {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::read_u64(input)?;

        Ok((input, Address(address)))
    }
}

impl Read<u32> for Address {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::read_u32(input)?;

        Ok((input, Address(address.into())))
    }
}

impl Read<u64> for Option<Address> {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = <Address as Read<u64>>::read::<N, E>(input)?;

        Ok((input, if address.0 == 0 { None } else { Some(address) }))
    }
}

impl Write<u64> for Address {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u64(self.0))
    }
}

impl Write<u32> for Address {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u32(
            self.0.try_into().expect("Failed to cast the alignment from `u64` to `u32`"),
        ))
    }
}

impl Write<u64> for Option<Address> {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        match self {
            Some(address) => <Address as Write<u64>>::write::<N, _>(address, buffer),
            None => buffer.write_all(&N::write_u64(0)),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "0x{:08x}", self.0)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0.try_into().unwrap()
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl Add for Address {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(
            self.0
                .checked_add(other.0)
                .ok_or_else(|| format!("`{self} + {other}` has overflowed"))
                .unwrap(),
        )
    }
}

/// An alignment value.
///
/// It's guaranteed to be a non-zero power of two, encoded in a `u64`.
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct Alignment(pub Option<NonZeroU64>);

impl Read for Alignment {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (next_input, alignment) = N::read_u64(input)?;

        let alignment = if alignment != 0 {
            if !alignment.is_power_of_two() {
                return Err(Error(E::from_error_kind(input, ErrorKind::Digit)));
            }

            // SAFETY: We just checked that there's no `0`.
            Some(unsafe { NonZeroU64::new_unchecked(alignment) })
        } else {
            None
        };

        Ok((next_input, Self(alignment)))
    }
}

impl Write for Alignment {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u64(self.0.map_or(0, NonZeroU64::get)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::BigEndian;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit_elf_amd64.o");

    #[test]
    fn test_address() {
        // From u64.
        assert_read_write!(
            Address: Read<u64> + Write<u64> {
                bytes_value(auto_endian) = 42u64,
                rust_value = Address(42),
            }
        );

        // From u32.
        assert_read_write!(
            Address: Read<u32> + Write<u32> {
                bytes_value(auto_endian) = 42u32,
                rust_value = Address(42),
            }
        );

        // As option: Some.
        assert_read_write!(
            Option<Address>: Read<u64> + Write<u64> {
                bytes_value(auto_endian) = 42u64,
                rust_value = Some(Address(42)),
            }
        );

        // As option: None.
        assert_read_write!(
            Option<Address>: Read<u64> + Write<u64> {
                bytes_value(auto_endian) = 0u64,
                rust_value = None,
            }
        );
    }

    #[test]
    fn test_alignment() {
        // No alignment.
        assert_read_write!(
            Alignment: Read<()> + Write<()> {
                bytes_value(auto_endian) = 0u64,
                rust_value = Alignment(None),
            }
        );

        // Some value alignment.
        assert_read_write!(
            Alignment: Read<()> + Write<()> {
                bytes_value(auto_endian) = 512u64,
                rust_value = Alignment(Some(NonZeroU64::new(512).unwrap())),
            }
        );

        // Some invalid (because not a power of two) alignment
        assert_eq!(
            Alignment::read::<BigEndian, ()>(&513u64.to_be_bytes()),
            Err(nom::Err::Error(())),
        );
    }

    #[test]
    fn test_me() {
        let (_remaining, mut file) = File::read::<()>(EXIT_FILE).unwrap();
        file.fetch_section_names();

        dbg!(&file);

        let strings_section = file.strings_section();

        for section in
            file.sections.iter().filter(|section| section.r#type == SectionType::SymbolTable)
        {
            let symbols = section.data.symbols::<()>(strings_section).unwrap().collect::<Vec<_>>();

            dbg!(&symbols);
        }
    }
}
