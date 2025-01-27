use std::{borrow::Cow, io};

use enumflags2::{bitflags, BitFlags};
use nom::Parser;
use weld_object_macros::ReadWrite;

use super::{Address, Alignment, Data, DataType};
use crate::{combinators::*, Input, Number, Read, Result, Write};

/// Program.
#[derive(Debug, PartialEq)]
pub struct Program<'a> {
    /// Identifies the type of the segment.
    pub r#type: ProgramType,
    /// Segment-dependent flags.
    pub segment_flags: ProgramFlags,
    /// Offset of the segment in the file image.
    pub offset: Address,
    /// Virtual address of the segment in memory.
    pub virtual_address: Address,
    /// On systems where physical address is relevant, reserved for segment's
    /// physical address.
    pub physical_address: Option<Address>,
    /// Size in bytes of the segment in the file image. May be 0.
    pub segment_size_in_file_image: Address,
    /// Size in bytes of the segment in memory. May be 0.
    pub segment_size_in_memory: Address,
    /// 0 and 1 specify no alignment. Otherwise should be a positive,
    /// integral power of 2, with `virtual_address` equating `offset` modulus
    /// `alignment`.
    pub alignment: Alignment,
    /// Data.
    pub data: Data<'a>,
}

impl<'a> Program<'a> {
    pub fn read<N, E>(input: Input<'a>, file: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (
            input,
            (
                r#type,
                segment_flags,
                offset,
                virtual_address,
                physical_address,
                segment_size_in_file_image,
                segment_size_in_memory,
                alignment,
            ),
        ) = (
            ProgramType::read::<N, _>,
            ProgramFlags::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <Option<Address> as Read<u64>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            Alignment::read::<N, _>,
        )
            .parse(input)?;

        let program = Self {
            r#type,
            offset,
            virtual_address,
            physical_address,
            segment_size_in_file_image,
            segment_size_in_memory,
            alignment,
            segment_flags,
            data: Data::new(
                Cow::Borrowed(
                    &file[offset.into()..][..segment_size_in_file_image.try_into().unwrap()],
                ),
                DataType::ProgramData,
                N::endianness(),
                None,
            ),
        };

        Ok((input, program))
    }
}

impl<'a> Write for Program<'a> {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        self.r#type.write::<N, _>(buffer)?;
        self.segment_flags.write::<N, _>(buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.offset, buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.virtual_address, buffer)?;
        <Option<Address> as Write<u64>>::write::<N, _>(&self.physical_address, buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.segment_size_in_file_image, buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.segment_size_in_memory, buffer)?;
        self.alignment.write::<N, _>(buffer)
    }
}

/// Type of program.
#[derive(ReadWrite, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProgramType {
    /// Program header table entry unused.
    Null = 0x00,
    /// Loadable segment.
    Load = 0x01,
    /// Dynamic linking information.
    Dynamic = 0x02,
    /// Interpreter information.
    Interpreter = 0x03,
    /// Auxiliary information.
    Note = 0x04,
    /// Reserved.
    Shlib = 0x05,
    /// Segment containing program header table itself.
    ProgramHeader = 0x06,
    /// Thread-Local Storage template.
    ThreadLocalStorage = 0x07,
}

/// Program flag.
#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProgramFlag {
    Execute = 0x1,
    Write = 0x2,
    Read = 0x4,
}

/// Program flags.
pub type ProgramFlags = BitFlags<ProgramFlag>;

impl Read for ProgramFlags {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, ProgramFlags, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = N::read_u32(input)?;
        let flags = ProgramFlags::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}

impl Write for ProgramFlags {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u32(self.bits()))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::*;
    use crate::{BigEndian, Endianness};

    #[test]
    fn test_program() {
        #[rustfmt::skip]
        let input: &[u8] = &[
            // Type.
            0x00, 0x00, 0x00, 0x01,
            // Flag.
            0x00, 0x00, 0x00, 0x05,
            // Offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Virtual address.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            // Physical address.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Segment size in file image.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
            // Segment size in memory.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Alignment.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
        ];

        let file: &[u8] = &[0x0, 0x61, 0x62, 0x63, 0x0];

        let program = Program {
            r#type: ProgramType::Load,
            offset: Address(0),
            virtual_address: Address(7),
            physical_address: None,
            segment_size_in_file_image: Address(5),
            segment_size_in_memory: Address(0),
            alignment: Alignment(Some(NonZeroU64::new(512).unwrap())),
            segment_flags: ProgramFlag::Read | ProgramFlag::Execute,
            data: Data::new(Cow::Borrowed(&file[..]), DataType::ProgramData, Endianness::Big, None),
        };

        let mut buffer = Vec::new();
        program.write::<BigEndian, _>(&mut buffer).unwrap();

        assert_eq!(buffer, input);

        assert_eq!(Program::read::<BigEndian, ()>(input, file), Ok((&[] as &[u8], program)));
    }

    #[test]
    fn test_program_flag() {
        macro_rules! test {
            ( $( $input:expr => $result:expr ),* $(,)? ) => {{
                $(
                    assert_read_write!(
                        ProgramFlags: Read<()> + Write<()> {
                            bytes_value(auto_endian) = $input as u32,
                            rust_value = ProgramFlags::from_bits($result as _).unwrap(),
                        }
                    );
                )*
            }};
        }

        test!(
            0x1 => ProgramFlag::Execute,
            0x2 => ProgramFlag::Write,
            0x4 => ProgramFlag::Read,
        );
    }
}
