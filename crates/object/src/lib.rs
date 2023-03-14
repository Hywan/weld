//! `weld-object` is able to read and write various object file binary formats,
//! like `elf64`.

#![deny(unused)]
#![deny(warnings)]
// #![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]

#[cfg(test)]
#[macro_use]
mod test;

mod combinators;
#[cfg(feature = "elf64")]
pub mod elf64;
mod endianness;
pub mod slice;
pub mod write;

pub use endianness::*;

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
