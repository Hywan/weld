//! The `Write` trait.
//!
//! This trait does the opposite of `read` to parse a binary format to a Rust
//! value. The `Write` trait compiles a Rust value into a binary format.

use std::io;

use crate::Number;

pub trait Write {
    /// Write part of `self` into the `buffer`.
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<usize>
    where
        N: Number,
        B: io::Write;

    /// Write part of `self` into the `buffer`, where `self` has been read from
    /// a `u32`.
    fn write_u32<N, B>(&self, buffer: &mut B) -> io::Result<usize>
    where
        N: Number,
        B: io::Write,
    {
        self.write::<N, B>(buffer)
    }

    /// Write part of `self` into the `buffer`, where `self` has been read from
    /// a `u16`.
    fn write_u16<N, B>(&self, buffer: &mut B) -> io::Result<usize>
    where
        N: Number,
        B: io::Write,
    {
        self.write::<N, B>(buffer)
    }
}
