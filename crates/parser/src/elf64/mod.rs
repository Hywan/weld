use crate::{generators::*, Input, Result};
use enumflags2::{bitflags, BitFlags};
use nom::number::complete::{le_u16, le_u32, le_u64};
use std::fmt;
use weld_parser_macros::EnumParse;

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Endianness {
    // Little.
    Little = 0x01,
    // Big
    Big = 0x02,
}

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Version {
    // Invalid version.
    None = 0x00,

    // Current version.
    Current = 0x01,
}

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OsAbi {
    /// [System V](https://en.wikipedia.org/wiki/System_V).
    SystemV = 0x00,
    /// [HP-UX](https://en.wikipedia.org/wiki/HP-UX).
    HpUx = 0x01,
    /// [NetBSD](https://en.wikipedia.org/wiki/NetBSD).
    NetBsd = 0x02,
    /// [GNU Linux](https://en.wikipedia.org/wiki/Linux).
    Gnu = 0x03,
    /// [GNU Hurd](https://en.wikipedia.org/wiki/GNU_Hurd).
    GnuHurd = 0x04,
    /// [Sun Solaris](https://en.wikipedia.org/wiki/Solaris_(operating_system)).
    Solaris = 0x06,
    /// [IBM AIX (Monterey)](https://en.wikipedia.org/wiki/IBM_AIX).
    Aix = 0x07,
    /// [SGI IRIX](https://en.wikipedia.org/wiki/IRIX).
    Irix = 0x08,
    /// [FreeBSD](https://en.wikipedia.org/wiki/FreeBSD).
    FreeBsd = 0x09,
    /// [Compaq TRU64 UNIX](https://en.wikipedia.org/wiki/Tru64).
    Tru64 = 0x0a,
    /// Novell Modesto.
    Modesto = 0x0b,
    /// [OpenBSD](https://en.wikipedia.org/wiki/OpenBSD).
    OpenBsd = 0x0c,
    /// [OpenVMS](https://en.wikipedia.org/wiki/OpenVMS).
    OpenVms = 0x0d,
    /// [Hewlett-Packard Non-Stop Kernel](https://en.wikipedia.org/wiki/NonStop_(server_computers)).
    Nsk = 0x0e,
    /// [AROS](https://en.wikipedia.org/wiki/AROS_Research_Operating_System).
    Aros = 0x0f,
    /// FenixOS.
    FenixOs = 0x10,
    /// [Nuxi CloudABI](https://en.wikipedia.org/wiki/CloudABI).
    CloudAbi = 0x11,
    /// [Stratus Technologies OpenVOS](https://en.wikipedia.org/wiki/Stratus_VOS).
    OpenVos = 0x12,
    /// ARM EABI.
    ArmAeabi = 0x40,
    /// ARM.
    Arm = 0x61,
    /// Standalone (embedded) application.
    Standalone = 0xff,
}

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FileType {
    /// Unknown.
    None = 0x00,
    /// Relocatable file.
    RelocatableFile = 0x01,
    /// Executable file.
    ExecutableFile = 0x02,
    /// Shared object.
    SharedObject = 0x03,
    /// Core file.
    CoreFile = 0x04,
}

/// Architecture.
#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Machine {
    /// No specific instruction set.
    None = 0x00,
    /// [AT&T WE 32100](https://en.wikipedia.org/wiki/Bellmac_32).
    Att32 = 0x01,
    /// [SPARC](https://en.wikipedia.org/wiki/SPARC).
    Sparc = 0x02,
    /// [x86](https://en.wikipedia.org/wiki/X86).
    X86 = 0x03,
    /// [Motorola 68000 (M68k)](https://en.wikipedia.org/wiki/Motorola_68000_series).
    Motorola68k = 0x04,
    /// [Motorola 88000 (M88k)](https://en.wikipedia.org/wiki/Motorola_88000).
    Motorola88k = 0x05,
    /// [Intel MCU](https://en.wikipedia.org/wiki/List_of_Intel_processors#Microcontrollers).
    IntelMcu = 0x06,
    /// [Intel 80860](https://en.wikipedia.org/wiki/Intel_i860).
    Intel860 = 0x07,
    /// [MIPS](https://en.wikipedia.org/wiki/MIPS_architecture).
    Mips = 0x08,
    /// [IBM System/370](https://en.wikipedia.org/wiki/IBM_System/370).
    IbmS370 = 0x09,
    /// [MIPS RS3000 Little-endian](https://en.wikipedia.org/wiki/R3000).
    MipsRs3Le = 0x0a,
    /// [Hewlett-Packard PA-RISC](https://en.wikipedia.org/wiki/PA-RISC).
    HpPaRisc = 0x0e,
    /// Fujitsu VPP500..
    FujitsuVpp500 = 0x11,
    /// Sun's “v8plus”.
    Sparc32Plus = 0x12,
    /// [Intel 80960](https://en.wikipedia.org/wiki/Intel_i960).
    Intel960 = 0x13,
    /// [PowerPC](https://en.wikipedia.org/wiki/PowerPC).
    PowerPc = 0x14,
    /// [PowerPC](https://en.wikipedia.org/wiki/PowerPC) (64-bit).
    PowerPc64 = 0x15,
    /// [S390](https://en.wikipedia.org/wiki/Z/Architecture), including S390x.
    IbmS390 = 0x16,
    /// IBM SPU/SPC.
    IbmSpu = 0x17,
    /// [NEC V800](https://en.wikipedia.org/wiki/V850).
    NecV800 = 0x24,
    /// Fujitsu FR20.
    FujitsuFr20 = 0x25,
    /// [TRW RH-32](https://en.wikipedia.org/wiki/RH-32).
    TrwRh32 = 0x26,
    /// Motorola RCE.
    MotorolaRce = 0x27,
    /// [Arm](https://en.wikipedia.org/wiki/ARM_architecture_family) (up to Armv7/AArch32).
    Arm = 0x28,
    /// [Digital Alpha](https://en.wikipedia.org/wiki/Digital_Alpha).
    DigitalAlpha = 0x29,
    /// [SuperH](https://en.wikipedia.org/wiki/SuperH).
    HitachiSuperH = 0x2a,
    /// [SPARC Version 9](https://en.wikipedia.org/wiki/SPARC).
    SparcV9 = 0x2b,
    /// [Siemens TriCore embedded processor](https://en.wikipedia.org/wiki/Infineon_TriCore).
    SiemensTricore = 0x2c,
    /// [Argonaut RISC Core](https://en.wikipedia.org/wiki/ARC_(processor)).
    ArgonautRiscCore = 0x2d,
    /// [Hitachi H8/300](https://en.wikipedia.org/wiki/H8_Family).
    HitachiH8300 = 0x2e,
    /// [Hitachi H8/300H](https://en.wikipedia.org/wiki/H8_Family).
    HitachiH8_300H = 0x2f,
    /// [Hitachi H8S](https://en.wikipedia.org/wiki/H8_Family).
    HitachiH8S = 0x30,
    /// [Hitachi H8/500](https://en.wikipedia.org/wiki/H8_Family).
    HitachiH8_500 = 0x31,
    /// [IA-64](https://en.wikipedia.org/wiki/IA-64).
    IntelA64 = 0x32,
    /// [Stanford MIPS-X](https://en.wikipedia.org/wiki/MIPS-X).
    MipsX = 0x33,
    /// [Motorola ColdFire](https://en.wikipedia.org/wiki/NXP_ColdFire).
    MotorolaColdfire = 0x34,
    /// [Motorola M68HC12](https://en.wikipedia.org/wiki/Motorola_68HC12).
    MotorolaM68Hc12 = 0x35,
    /// Fujitsu MMA Multimedia Accelerator.
    FujitsuMma = 0x36,
    /// Siemens PCP.
    SiemensPcp = 0x37,
    /// [Sony nCPU embedded RISC processor](https://en.wikipedia.org/wiki/Cell_(microprocessor)).
    SonuNcpu = 0x38,
    /// Denso NDR1 microprocessor.
    DensoNdr1 = 0x39,
    /// Motorola Star\*Core processor.
    MotorolaStarCore = 0x3a,
    /// Toyota ME16 processor.
    ToyotaMe16 = 0x3b,
    /// STMicroelectronics ST100 processor.
    St100 = 0x3c,
    /// Advanced Logic Corp. TinyJ embedded processor family.
    TinyJ = 0x3d,
    /// [AMD x86-64](https://en.wikipedia.org/wiki/Amd64).
    X86_64 = 0x3e,
    /// Sony DSP Processor.
    SonyDSP = 0x3f,
    /// [Digital Equipment Corp. PDP-10](https://en.wikipedia.org/wiki/PDP-10).
    Pdp10 = 0x40,
    /// [Digital Equipment Corp. PDP-11](https://en.wikipedia.org/wiki/PDP-11).
    Pdp11 = 0x41,
    /// Siemens FX66 microcontroller.
    SiemensFx66 = 0x42,
    /// STMicroelectronics ST9+ 8/16 bit microcontroller.
    St9Plus = 0x43,
    /// STMicroelectronics ST7 8-bit microcontroller.
    St7 = 0x44,
    /// [Motorola MC68HC16 Microcontroller](https://en.wikipedia.org/wiki/Motorola_68HC16).
    Motorola68Hc16 = 0x45,
    /// [Motorola MC68HC11 Microcontroller](https://en.wikipedia.org/wiki/Motorola_68HC11).
    Motorola68Hc11 = 0x46,
    /// [Motorola MC68HC08 Microcontroller](https://en.wikipedia.org/wiki/Motorola_68HC08).
    Motorola68Hc08 = 0x47,
    /// [Motorola MC68HC05 Microcontroller](https://en.wikipedia.org/wiki/Motorola_68HC05).
    Motorola68Hc05 = 0x48,
    /// Silicon Graphics SVx.
    Svx = 0x49,
    /// STMicroelectronics ST19 8-bit microcontroller.
    St19 = 0x4a,
    /// [Digital VAX](https://en.wikipedia.org/wiki/VAX).
    Vax = 0x4b,
    /// Axis Communications 32-bit embedded processor.
    Cris = 0x4c,
    /// Infineon Technologies 32-bit embedded processor.
    Javelin = 0x4d,
    /// Element 14 64-bit DSP Processor.
    FirePath = 0x4e,
    /// LSI Logic 16-bit DSP Processor.
    Zsp = 0x4f,
    /// [TMS320C6000 Family](https://en.wikipedia.org/wiki/Texas_Instruments_TMS320#C6000_series).
    TiC6000 = 0x8c,
    /// [MCST Elbrus e2k](https://en.wikipedia.org/wiki/Elbrus_2000).
    McstElbrus = 0xaf,
    /// [Arm 64-bits](https://en.wikipedia.org/wiki/AArch64) (Armv8/AArch64).
    Aarch64 = 0xb7,
    /// [RISC-V](https://en.wikipedia.org/wiki/RISC-V).
    RiscV = 0xf3,
    /// [Berkeley Packet Filter](https://en.wikipedia.org/wiki/Berkeley_Packet_Filter).
    Bpf = 0xf7,
}

#[repr(transparent)]
pub struct Address(pub u64);

impl Address {
    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, address) = le_u64(input)?;

        Ok((input, Address(address)))
    }

    pub fn maybe_parse<'a, E>(input: Input<'a>) -> Result<Option<Self>, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, address) = Self::parse(input)?;

        Ok((input, if address.0 == 0 { None } else { Some(address) }))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "0x{:08x}", self.0)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

impl Into<usize> for Address {
    fn into(self) -> usize {
        self.0 as usize
    }
}

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

#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProgramFlag {
    Execute = 0x1,
    Write = 0x2,
    Read = 0x4,
}

pub type ProgramFlags = BitFlags<ProgramFlag>;

impl ProgramFlag {
    pub fn parse_bits<'a, E>(input: Input<'a>) -> Result<ProgramFlags, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = le_u32(input)?;
        let flags = ProgramFlags::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}

#[derive(Debug)]
pub struct ProgramHeader {
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
    pub alignment: u64,
}

impl ProgramHeader {
    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
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
            ProgramType::parse,
            ProgramFlag::parse_bits,
            Address::parse,
            Address::parse,
            Address::maybe_parse,
            le_u64,
            le_u64,
            le_u64,
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
        };

        Ok((input, program_header))
    }
}

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SectionType {
    /// Section header table entry unused.
    Null = 0x00,
    /// Program data.
    ProgramData = 0x01,
    /// Symbol table.
    SymbolTable = 0x02,
    /// String table.
    StringTable = 0x03,
    /// Relocation entries with addends.
    RelocationWithAddends = 0x04,
    /// Symbol hash table.
    SymbolHashTable = 0x05,
    /// Dynamic linking information.
    DynamicLinkingInformation = 0x06,
    /// Notes.
    Note = 0x07,
    /// Program space with no data (BSS, Block Started by Symbol).
    NoBits = 0x08,
    /// Relocation entries, no addends.
    Relocation = 0x09,
    /// Reserved.
    Shlib = 0x0a,
    /// Dynamic linker symbol table.
    DynamicLinkerSysmbolTable = 0x0b,
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
}

#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
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

pub type SectionFlags = BitFlags<SectionFlag>;

impl SectionFlag {
    pub fn parse_bits<'a, E>(input: Input<'a>) -> Result<SectionFlags, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, flags) = le_u32(input)?;
        let flags = SectionFlags::from_bits(flags)
            .map_err(|_| Err::Error(E::from_error_kind(input, ErrorKind::Alt)))?;

        Ok((input, flags))
    }
}

#[derive(Debug)]
pub struct SectionHeader {
    /// An offset to a string in the `.shstrtab` section that represents the
    /// name of this section.
    pub name: u32,
    /// Type of the section header.
    pub r#type: SectionType,
    /// Flags.
    pub flags: SectionFlags,
    pub addr: Address,
    pub offset: Address,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entsize: u64,
}

impl SectionHeader {
    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, (name, r#type, flags, addr, offset, size, link, info, addralign, entsize)) =
            tuple((
                le_u32,
                SectionType::parse,
                SectionFlag::parse_bits,
                Address::parse,
                Address::parse,
                le_u64,
                le_u32,
                le_u32,
                le_u64,
                le_u64,
            ))(input)?;

        let section_header = Self {
            name,
            r#type,
            flags,
            addr,
            offset,
            size,
            link,
            info,
            addralign,
            entsize,
        };

        Ok((input, section_header))
    }
}

#[derive(Debug)]
pub struct FileHeader {
    /// Endianess of the object file.
    pub endianness: Endianness,

    /// Object file version.
    pub version: Version,

    /// OS ABI.
    pub os_abi: OsAbi,

    /// Object file type.
    pub r#type: FileType,

    /// Machine architecture.
    pub machine: Machine,

    /// Processor-specific flags.
    pub processor_flags: u32,

    /// Entry point virtual address.
    pub entry_point: Option<Address>,

    /// Program headers.
    pub program_headers: Vec<ProgramHeader>,

    /// Section headers.
    pub section_headers: Vec<SectionHeader>,
}

impl FileHeader {
    const MAGIC: &'static [u8; 4] = &[0x7f, b'E', b'L', b'F'];
    const ELF64: &'static [u8; 1] = &[0x2];

    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let file = input;

        // `fh` stands for `file_header`.
        // `ph` stands for `program_header`.
        // `sh` stands for `section_header`.

        let (
            input,
            (
                _magic,
                _class,
                endianness,
                version,
                os_abi,
                _padding,
                r#type,
                machine,
                _version_bis,
                entry_point,
                ph_offset,
                sh_offset,
                processor_flags,
                _fh_size,
                ph_entry_size,
                ph_number,
                sh_entry_size,
                sh_number,
                section_name_index,
            ),
        ) = tuple((
            tag(Self::MAGIC),
            tag(Self::ELF64),
            Endianness::parse,
            Version::parse,
            OsAbi::parse,
            skip(8usize),
            FileType::parse,
            Machine::parse,
            skip(4usize),
            Address::maybe_parse,
            Address::parse,
            Address::parse,
            le_u32,
            skip(2usize),
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
        ))(input)?;

        let mut program_headers = Vec::with_capacity(ph_number as usize);

        if ph_entry_size > 0 {
            for ph_slice in (&file[ph_offset.into()..])
                .chunks_exact(ph_entry_size as usize)
                .take(ph_number as usize)
            {
                let (_, ph) = ProgramHeader::parse(ph_slice)?;
                program_headers.push(ph);
            }
        }

        let mut section_headers = Vec::with_capacity(sh_number as usize);

        if sh_entry_size > 0 {
            for sh_slice in (&file[sh_offset.into()..])
                .chunks_exact(sh_entry_size as usize)
                .take(sh_number as usize)
            {
                let (_, sh) = SectionHeader::parse(sh_slice)?;
                section_headers.push(sh);
            }
        }

        let file_header = Self {
            endianness,
            version,
            os_abi,
            r#type,
            machine,
            processor_flags,
            entry_point,
            program_headers,
            section_headers,
        };

        Ok((input, file_header))
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit.elfx86_64.o");

    #[test]
    fn test_me() {
        let (remaining, file) = FileHeader::parse::<VerboseError<Input>>(EXIT_FILE).unwrap();
        // dbg!(&remaining);
        dbg!(&file);
    }
}