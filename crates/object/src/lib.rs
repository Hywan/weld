//! `weld-object` is able to read and write various object file binary formats,
//! like `elf64`.

// TODO: remove
#![allow(missing_docs)]

#[cfg(test)]
#[macro_use]
mod test;

mod combinators;
#[cfg(feature = "elf64")]
pub mod elf64;
mod endianness;
mod read_write;

pub use endianness::*;
pub use read_write::{Read, Write};

/// Represent the input type of the parsers.
pub type Input<'a> = &'a [u8];

/// Represent the result returned by the parsers.
pub type Result<'a, O, E> = nom::IResult<Input<'a>, O, E>;

/// Errors used by the crate.
pub mod errors {
    pub use nom::Err as Error;

    /// Represent an error that can be used by parser, which doesn't accumulate
    /// multiple errors, but stores just one.
    pub type SingleError<'a> = nom::error::Error<super::Input<'a>>;

    pub use nom::error::ErrorKind;
}
