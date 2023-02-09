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
#[derive(Copy, Clone)]
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
#[derive(Debug)]
#[repr(transparent)]
pub struct Alignment(Option<NonZeroU64>);

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

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit_elf_amd64");

    #[test]
    fn test_me() {
        let (_remaining, file) = File::parse::<VerboseError<Input>>(EXIT_FILE).unwrap();
        // dbg!(&remaining);
        dbg!(&file);

        let symbol_section = file
            .sections
            .iter()
            .find(|section| section.r#type == SectionType::SymbolTable)
            .unwrap();

        for symbol in symbol_section.data.symbols::<VerboseError<Input>>().unwrap() {
            let symbol = symbol.unwrap();
            dbg!(&symbol);
        }
    }
}
