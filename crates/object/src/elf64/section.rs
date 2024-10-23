use std::{borrow::Cow, io, num::NonZeroU64};

use bstr::BStr;
use enumflags2::{bitflags, BitFlags};
use weld_object_macros::ReadWrite;

use super::{Address, Alignment, Data};
use crate::{combinators::*, Input, Number, Read, Result, Write};

/// Section header.
#[derive(Debug, PartialEq)]
pub struct Section<'a> {
    /// Name of the section, if any.
    pub name: Option<Cow<'a, BStr>>,
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
    pub link: SectionIndex,
    /// Contains extra information about the section. This field is used for
    /// several purposes, depending on the type of section.
    pub information: u32,
    /// Contains the required alignment of the section.
    pub alignment: Alignment,
    /// Contains some size, in bytes, of each entry, for sections that contain
    /// fixed-sized entries.
    pub entity_size: Option<NonZeroU64>,
    /// Data.
    pub data: Data<'a>,
}

impl<'a> Section<'a> {
    pub fn read<N, E>(input: Input<'a>, file: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
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
            <Address as Read<u32>>::read::<N, _>,
            SectionType::read::<N, _>,
            SectionFlags::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            <SectionIndex as Read<u32>>::read::<N, _>,
            N::read_u32,
            Alignment::read::<N, _>,
            N::read_u64,
        ))(input)?;

        let entity_size = if entity_size != 0 {
            // SAFETY: We just checked `entity_size` is not 0.
            Some(unsafe { NonZeroU64::new_unchecked(entity_size) })
        } else {
            None
        };

        let section = Self {
            name: None,
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
            data: Data::new(
                Cow::Borrowed(&file[offset.into()..][..segment_size_in_file_image.into()]),
                r#type.into(),
                N::endianness(),
                entity_size,
            ),
        };

        Ok((input, section))
    }
}

impl<'a> Write for Section<'a> {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        <Address as Write<u32>>::write::<N, _>(&self.name_offset, buffer)?;
        self.r#type.write::<N, _>(buffer)?;
        self.flags.write::<N, _>(buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.virtual_address, buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.offset, buffer)?;
        <Address as Write<u64>>::write::<N, _>(&self.segment_size_in_file_image, buffer)?;
        <SectionIndex as Write<u32>>::write::<N, _>(&self.link, buffer)?;
        buffer.write_all(&N::write_u32(self.information))?;
        self.alignment.write::<N, _>(buffer)?;
        buffer.write_all(&N::write_u64(self.entity_size.map_or(0, NonZeroU64::get)))
    }
}

/// Section type.
#[derive(ReadWrite, Debug, Clone, Copy, PartialEq, Eq)]
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
    DynamicLinkingTable = 0x06,
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
    DynamicLoaderSymbolTable = 0x0b,
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
    /// The section contains writable data.
    Writable = 0x01,
    /// The section is allocated in memory image of program.
    Allocable = 0x02,
    /// The section contains executable instructions.
    Executable = 0x04,
    /// The sectionn might be merged.
    Merge = 0x10,
    /// The section contains null-terminated strings.
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
    // Disabled because those are not powers of two, then it's incompatible with `#[bitflags]`.
    //
    // /// Environment-specific use.
    // EnvironmentSpecific = 0x0f00_0000,
    // /// Processor-specific use.
    // ProcessorSpecific = 0xf000_0000,
}

/// Section flags.
pub type SectionFlags = BitFlags<SectionFlag>;

impl Read for SectionFlags {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = N::read_u64(input)?;
        let flags = Self::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}

impl Write for SectionFlags {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u64(self.bits()))
    }
}

/// Section index.
#[derive(Debug, PartialEq, Eq)]
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
    fn _read<'a, E>(input: Input<'a>, index: u32) -> Result<'a, Self, E>
    where
        E: ParseError<Input<'a>>,
    {
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
                index => Self::Ok(
                    index
                        .try_into()
                        .expect("Failed to cast the section index from `u32` to `usize`"),
                ),
            },
        ))
    }
}

impl Read<u32> for SectionIndex {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, index) = N::read_u32(input)?;

        Self::_read(input, index)
    }
}

impl Read<u16> for SectionIndex {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, index) = N::read_u16(input)?;

        Self::_read(input, index.into())
    }
}

macro_rules! section_index_write {
    ($section_index:ident) => {
        match $section_index {
            SectionIndex::Undefined => 0x0000,
            SectionIndex::LowProcessorSpecific => 0xff00,
            SectionIndex::HighProcessorSpecific => 0xff1f,
            SectionIndex::LowEnvironmentSpecific => 0xff20,
            SectionIndex::HighEnvironmentSpecific => 0xff3f,
            SectionIndex::Absolute => 0xfff1,
            SectionIndex::Common => 0xfff2,
            SectionIndex::Ok(index) => {
                (*index).try_into().expect("Failed to cast the section index from `usize` to `u32`")
            }
        }
    };
}

impl Write<u32> for SectionIndex {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u32(section_index_write!(self)))
    }
}

impl Write<u16> for SectionIndex {
    fn write<N, B>(&self, buffer: &mut B) -> io::Result<()>
    where
        N: Number,
        B: io::Write,
    {
        buffer.write_all(&N::write_u16(section_index_write!(self)))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::DataType, *};
    use crate::{BigEndian, Endianness};

    #[test]
    fn test_section() {
        #[rustfmt::skip]
        let input: &[u8] = &[
            // Name offset.
            0x00, 0x00, 0x00, 0x01,
            // Type.
            0x00, 0x00, 0x00, 0x03,
            // Flag.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Virtual address.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            // Offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Segment size in file image.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
            // Link.
            0x00, 0x00, 0x00, 0x03,
            // Information.
            0x00, 0x00, 0x00, 0x00,
            // Alignment.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
            // Entity size.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let file: &[u8] = &[0x0, 0x61, 0x62, 0x63, 0x0];

        let section = Section {
            name: None,
            name_offset: Address(1),
            r#type: SectionType::StringTable,
            flags: SectionFlags::EMPTY,
            virtual_address: Address(7),
            offset: Address(0),
            segment_size_in_file_image: Address(5),
            link: SectionIndex::Ok(3),
            information: 0,
            alignment: Alignment(Some(NonZeroU64::new(512).unwrap())),
            entity_size: None,
            data: Data::new(Cow::Borrowed(&file[..]), DataType::StringTable, Endianness::Big, None),
        };

        let mut buffer = Vec::new();
        section.write::<BigEndian, _>(&mut buffer).unwrap();

        assert_eq!(buffer, input);

        assert_eq!(Section::read::<BigEndian, ()>(input, file), Ok((&[] as &[u8], section)));
    }

    #[test]
    fn test_section_flag() {
        macro_rules! test {
            ( $( $input:expr => $result:expr ),* $(,)? ) => {{
                $(
                    assert_read_write!(
                        SectionFlags: Read<()> + Write<()> {
                            bytes_value(auto_endian) = $input as u64,
                            rust_value = SectionFlags::from_bits($result as _).unwrap(),
                        }
                    );
                )*
            }};
        }

        test!(
            0x01 => SectionFlag::Writable,
            0x02 => SectionFlag::Allocable,
            0x04 => SectionFlag::Executable,
            0x10 => SectionFlag::Merge,
            0x20 => SectionFlag::Strings,
            0x80 => SectionFlag::LinkOrder,
            0x100 => SectionFlag::OsNonConforming,
            0x200 => SectionFlag::IsPartOfAGroup,
            0x400 => SectionFlag::HasThreadLocalData,
        );
    }

    #[test]
    fn test_section_index() {
        macro_rules! test {
            ( $( $input:expr => $result:expr ),* $(,)? ) => {{
                $(
                    assert_read_write!(
                        SectionIndex: Read<u16> + Write<u16> {
                            bytes_value(auto_endian) = $input as u16,
                            rust_value = $result,
                        }
                    );
                    assert_read_write!(
                        SectionIndex: Read<u32> + Write<u32> {
                            bytes_value(auto_endian) = $input as u32,
                            rust_value = $result,
                        }
                    );
                )*
            }};
        }

        test!(
            0x0000 => SectionIndex::Undefined,
            0xff00 => SectionIndex::LowProcessorSpecific,
            0xff1f => SectionIndex::HighProcessorSpecific,
            0xff20 => SectionIndex::LowEnvironmentSpecific,
            0xff3f => SectionIndex::HighEnvironmentSpecific,
            0xfff1 => SectionIndex::Absolute,
            0xfff2 => SectionIndex::Common,
            0x0001 => SectionIndex::Ok(1),
            0x002a => SectionIndex::Ok(42),
        );
    }
}
