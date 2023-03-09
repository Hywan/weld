//! The `Write` trait.
//!
//! This trait does the opposite of `read` to parse a binary format to a Rust
//! value. The `Write` trait compiles a Rust value into a binary format.

use std::io;

use crate::{combinators::ParseError, Input, Number};

pub trait Write<'a, N, E, B>
where
    N: Number<'a, E>,
    E: ParseError<Input<'a>>,
    B: io::Write,
{
    /// Write part of `self` into the `buffer`.
    fn write(&self, buffer: &mut B) -> io::Result<usize>;
}
