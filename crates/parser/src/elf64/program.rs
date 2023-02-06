use enumflags2::{bitflags, BitFlags};
use weld_parser_macros::EnumParse;

use super::{Address, Alignment, Data};
use crate::{combinators::*, Input, Result};

/// Program.
#[derive(Debug)]
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
    pub segment_size_in_file_image: u64,
    /// Size in bytes of the segment in memory. May be 0.
    pub segment_size_in_memory: u64,
    /// 0 and 1 specify no alignment. Otherwise should be a positive,
    /// integral power of 2, with `virtual_address` equating `offset` modulus
    /// `alignment`.
    pub alignment: Alignment,
    /// Data.
    pub data: Data<'a>,
}

impl<'a> Program<'a> {
    pub fn parse<N, E>(file: Input<'a>, input: Input<'a>) -> Result<'a, Self, E>
    where
        N: NumberParser<'a, E>,
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
        ) = tuple((
            ProgramType::parse::<N, _>,
            ProgramFlag::parse_bits::<N, _>,
            Address::parse::<N, _>,
            Address::parse::<N, _>,
            Address::maybe_parse::<N, _>,
            N::u64,
            N::u64,
            Alignment::parse::<N, _>,
        ))(input)?;

        let program_header = Self {
            r#type,
            offset,
            virtual_address,
            physical_address,
            segment_size_in_file_image,
            segment_size_in_memory,
            alignment,
            segment_flags,
            data: Data::new(
                &file[offset.into()..][..segment_size_in_file_image.try_into().unwrap()],
            ),
        };

        Ok((input, program_header))
    }
}

/// Type of program.
#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
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
    ThreadLocalStraoge = 0x07,
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

impl ProgramFlag {
    pub fn parse_bits<'a, N, E>(input: Input<'a>) -> Result<ProgramFlags, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = N::u32(input)?;
        let flags = ProgramFlags::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}
