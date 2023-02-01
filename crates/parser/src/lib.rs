mod combinators;

#[cfg(feature = "elf64")]
pub mod elf64;

pub use combinators::NumberParser;

pub type Input<'a> = &'a [u8];
pub type Result<'a, O, E> = nom::IResult<Input<'a>, O, E>;
