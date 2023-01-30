pub mod elf64;
mod generators;

pub use generators::NumberParser;

pub type Input<'a> = &'a [u8];
pub type Result<'a, O, E> = nom::IResult<Input<'a>, O, E>;
