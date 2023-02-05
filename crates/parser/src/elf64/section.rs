use bstr::BStr;
use enumflags2::{bitflags, BitFlags};
use weld_parser_macros::EnumParse;

use super::{Address, Data};
use crate::{combinators::*, Input, Result};

/// Section header.
#[derive(Debug)]
pub struct Section<'a> {
    /// Name of the section, if any.
    pub name: Option<&'a BStr>,
    /// An offset to a string in the `.shstrtab` section that represents the
    /// name of this section.
    pub(super) name_offset: Address,
    /// Type of the section header.
    pub r#type: SectionType,
    /// Flags.
    pub flags: SectionFlags,
    /// Virtual address of the section in memory, for sections that are loaded.
    pub virtual_address: Address,
    /// Offset of the section in the file image.
    pub offset: Address,
    /// Size in bytes of the section in the file image. May be 0.
    pub segment_size_in_file_image: Address,
    /// Contains the section index of an associated section. This field is used
    /// for several purposes, depending on the type of section.
    pub link: u32,
    /// Contains extra information about the section. This field is used for
    /// several purposes, depending on the type of section.
    pub information: u32,
    /// Contains the required alignment of the section. This field must be a
    /// power of two.
    pub alignment: u64,
    /// Contains some size, in bytes, of each entry, for sections that contain
    /// fixed-sized entries.
    pub entity_size: Option<u64>,
    /// Data.
    pub data: Data<'a>,
}

impl<'a> Section<'a> {
    pub fn parse<N, E>(file: Input<'a>, input: Input<'a>) -> Result<'a, Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (
            input,
            (
                name_offset,
                r#type,
                flags,
                virtual_address,
                offset,
                segment_size_in_file_image,
                link,
                information,
                alignment,
                entity_size,
            ),
        ) = tuple((
            N::u32,
            SectionType::parse::<N, _>,
            SectionFlag::parse_bits::<N, _>,
            Address::parse::<N, _>,
            Address::parse::<N, _>,
            Address::parse::<N, _>,
            N::u32,
            N::u32,
            N::u64,
            N::u64,
        ))(input)?;

        let section_header = Self {
            name: None,
            name_offset: name_offset.into(),
            r#type,
            flags,
            virtual_address,
            offset,
            segment_size_in_file_image,
            link,
            information,
            alignment,
            entity_size: if entity_size == 0 { None } else { Some(entity_size) },
            data: Data::new(&file[offset.into()..][..segment_size_in_file_image.into()]),
        };

        Ok((input, section_header))
    }
}

/// Section type.
#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SectionType {
    /// Mark an unused section header.
    Null = 0x00,
    /// The section contains information defined by the program.
    ProgramData = 0x01,
    /// The section contains a linker symbol table.
    SymbolTable = 0x02,
    /// The section contains a string table.
    StringTable = 0x03,
    /// The seciton contains “Rela” type relocation entries, with addends (hence
    /// the “a” in “Rela”, i.e. “RELocations with Addends”).
    RelocationWithAddends = 0x04,
    /// The section contains a symbol hash table.
    SymbolHashTable = 0x05,
    /// The section contains dynamic linking tables.
    DynamicLinkingInformation = 0x06,
    /// The section contains note information.
    Note = 0x07,
    /// The section contains uninitialized space; does not occupy any space in
    /// the file. It represents program space with no data (BSS, Block
    /// Started by Symbol).
    NoBits = 0x08,
    /// The section contains “Rel” type relocation entries, without addends.
    Relocation = 0x09,
    /// Reserved.
    Shlib = 0x0a,
    /// The section contains a dynamic loader symbol table.
    DynamicLinkerSymbolTable = 0x0b,
    /// Array of constructors.
    ArrayOfConstructors = 0x0e,
    /// Array of destructors.
    ArrayOfDestructors = 0x0f,
    /// Array of pre-constructors.
    ArrayOfPreConstructors = 0x10,
    /// Section group.
    Group = 0x11,
    /// Extended section indices.
    ExtendedSectionIndices = 0x12,
    /// Number of defined types.
    NumberOfDefinedTypes = 0x13,
    /// Low environment-specific use.
    LowEnvironmentSpecific = 0x6000_0000,
    /// High environment-specific use.
    HighEnvironmentSpecific = 0x6fff_ffff,
    /// Low processor-specific use.
    LowProcessorSpecific = 0x7000_0000,
    /// High processor-specific use.
    HighProcessorSpecific = 0x7fff_ffff,
}

/// Section flag.
#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SectionFlag {
    /// Writable.
    Writable = 0x01,
    /// Occupies memory during execution.
    Allocable = 0x02,
    /// Executable.
    Executale = 0x04,
    /// Might be merged.
    Merge = 0x10,
    /// Contains null-terminated strings.
    Strings = 0x20,
    /// `sh_info` contains SHT index.
    InfoLink = 0x40,
    /// Preserve order after combining.
    LinkOrder = 0x80,
    /// Non-standard OS specific handling required.
    OsNonConforming = 0x100,
    /// Section is member of a group.
    IsPartOfAGroup = 0x200,
    /// Section hold thread-local data.
    HasThreadLocalData = 0x400,
}

/// Section flags.
pub type SectionFlags = BitFlags<SectionFlag>;

impl SectionFlag {
    pub fn parse_bits<'a, N, E>(input: Input<'a>) -> Result<SectionFlags, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = N::u64(input)?;
        let flags = SectionFlags::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}

pub enum SectionIndex {
    /// A valid section index.
    Ok(usize),
    /// An undefined or meaningless section reference.
    Undefined,
    /// Processor-specific use.
    LowProcessorSpecific,
    /// Processor-specific use.
    HighProcessorSpecific,
    /// Environment-specific use.
    LowEnvironmentSpecific,
    /// Environment-specific use.
    HighEnvironmentSpecific,
    /// The corresponding reference is an absolute value.
    Absolute,
    /// A symbol that has been declared as a common block (Fortran COMMON or C
    /// tentative declaration).
    Common,
}

impl SectionIndex {
    pub fn parse<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, index) = N::u16(input)?;

        Ok((
            input,
            match index {
                0x0000 => Self::Undefined,
                0xff00 => Self::LowProcessorSpecific,
                0xff1f => Self::HighProcessorSpecific,
                0xff20 => Self::LowEnvironmentSpecific,
                0xff3f => Self::HighEnvironmentSpecific,
                0xfff1 => Self::Absolute,
                0xfff2 => Self::Common,
                index => Self::Ok(index.into()),
            },
        ))
    }
}
