use std::{fmt, num::NonZeroU64, ops::Add};

use nom::Err::Error;

use crate::{combinators::*, Input, NumberParser, Result};

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
    pub fn parse<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::u64(input)?;

        Ok((input, Address(address)))
    }

    pub fn parse_u32<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = N::u32(input)?;

        Ok((input, Address(address.into())))
    }

    pub fn maybe_parse<'a, N, E>(input: Input<'a>) -> Result<Option<Self>, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, address) = Self::parse::<N, E>(input)?;

        Ok((input, if address.0 == 0 { None } else { Some(address) }))
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
    pub fn parse<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (next_input, alignment) = N::u64(input)?;

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

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;
    use crate::BigEndian;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit_elf_amd64");

    #[test]
    fn test_alignment() {
        // No alignment.
        assert_eq!(
            Alignment::parse::<BigEndian, ()>(&0u64.to_be_bytes()),
            Ok((&[] as &[u8], Alignment(None)))
        );

        // Some valid alignment.
        assert_eq!(
            Alignment::parse::<BigEndian, ()>(&512u64.to_be_bytes()),
            Ok((&[] as &[u8], Alignment(Some(NonZeroU64::new(512).unwrap()))))
        );

        // Some invalid (because not a power of two) alignment
        assert_eq!(
            Alignment::parse::<BigEndian, ()>(&513u64.to_be_bytes()),
            Err(nom::Err::Error(())),
        );
    }

    #[test]
    fn test_me() {
        let (remaining, file) = File::parse::<VerboseError<Input>>(EXIT_FILE).unwrap();
        dbg!(&file);

        let string_section = file.sections.iter().find(|section| matches!(section, Section { r#type: SectionType::StringTable, name: Some(section_name), .. } if *section_name == ".strtab")).unwrap();

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
