//! The `Read` and `Write` traits.

use std::io;

use crate::{combinators::ParseError, Input, Number, Result};

pub trait Read<Type = ()>
where
    Self: Sized,
{
    fn read<'r, N, E>(input: Input<'r>) -> Result<'r, Self, E>
    where
        N: Number,
        E: ParseError<Input<'r>>;
}

pub trait Write<ReadFrom = ()> {
    /// Write part of `self` into the `buffer`.
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write;
}
