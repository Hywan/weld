use std::{fmt, ops::Add};

use crate::{combinators::*, Input, Result};

mod data;
mod file;
mod program;
mod section;

pub use data::*;
pub use file::*;
pub use program::*;
pub use section::*;

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

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit.elfx86_64.o");

    #[test]
    fn test_me() {
        let (_remaining, file) = FileHeader::parse::<VerboseError<Input>>(EXIT_FILE).unwrap();
        // dbg!(&remaining);
        dbg!(&file);
    }
}
