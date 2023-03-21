use std::{fmt, io, num::NonZeroU64, ops::Add};

use nom::Err::Error;

use crate::{combinators::*, write::Write, Input, Number, Result};

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

impl Address {
    pub fn read<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::read_u64(input)?;

        Ok((input, Address(address)))
    }

    pub fn read_u32<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::read_u32(input)?;

        Ok((input, Address(address.into())))
    }

    pub fn maybe_read<'a, N, E>(input: Input<'a>) -> Result<Option<Self>, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = Self::read::<N, E>(input)?;

        Ok((input, if address.0 == 0 { None } else { Some(address) }))
    }
}

impl Write for Address {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<usize>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write(&N::write_u64(self.0))
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

impl Alignment {
    pub fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
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
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<usize>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write(&match self.0 {
            Some(alignment) => N::write_u64(alignment.get()),
            None => N::write_u64(0u64),
        })
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;
    use crate::BigEndian;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit_elf_amd64.o");

    #[test]
    fn test_address_read() {
        assert_read_write!(Address::read(42u64) <=> Address(42));
        assert_read_write!(Address::read_u32(42u32 ~ 42u64) <=> Address(42));
        assert_read!(Address::maybe_read(42u64) <=> Some(Address(42)));
        assert_read!(Address::maybe_read(0u64) <=> None);
    }

    #[test]
    fn test_alignment() {
        // No alignment.
        assert_read_write!(Alignment::read(0u64) <=> Alignment(None));

        // Some value alignment.
        assert_read_write!(
            Alignment::read(512u64) <=> Alignment(Some(NonZeroU64::new(512).unwrap()))
        );

        // Some invalid (because not a power of two) alignment
        assert_eq!(
            Alignment::read::<BigEndian, ()>(&513u64.to_be_bytes()),
            Err(nom::Err::Error(())),
        );
    }

    #[test]
    fn test_me() {
        let (_remaining, file) = File::read::<VerboseError<Input>>(EXIT_FILE).unwrap();
        dbg!(&file);

        let string_section = file
            .sections
            .iter()
            .find(|section| {
                matches!(
                    section,
                    Section {
                        r#type: SectionType::StringTable,
                        name: Some(section_name), ..
                    } if *section_name.as_ref() == ".strtab"
                )
            })
            .expect("`.strtab` section not found");

        for section in
            file.sections.iter().filter(|section| section.r#type == SectionType::SymbolTable)
        {
            let symbols = section
                .data
                .symbols::<VerboseError<Input>>()
                .unwrap()
                .map(|symbol| symbol.unwrap())
                .map(|mut symbol| {
                    symbol.name = string_section.data.string_at_offset(symbol.name_offset.into());
                    symbol
                })
                .collect::<Vec<_>>();

            dbg!(&symbols);
        }
    }
}
