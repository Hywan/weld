use crate::{Input, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    error::ParseError,
    sequence::tuple,
};

#[derive(Debug)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, endianness) = alt((tag(&[0x1]), tag(&[0x2])))(input)?;

        Ok((
            input,
            match endianness[0] {
                0x1 => Self::Little,
                0x2 => Self::Big,
                _ => unreachable!(),
            },
        ))
    }
}

#[derive(Debug)]
pub enum OsAbi {
    // Unknown OS ABI.
    None,

    // System V.
    SystemV,

    // HP-UX.
    HpUx,

    // NetBSD.
    NetBsd,

    // GNU.
    Gnu,

    // GNU/Hurd.
    GnuHurd,

    // Sun Solaris.
    Solaris,

    // IBM AIX.
    Aix,

    // SGI Irix.
    Irix,

    // FreeBSD.
    FreeBsd,

    // Compaq TRU64 UNIX.
    Tru64,

    // Novell Modesto.
    Modesto,

    // OpenBSD.
    OpenBsd,

    // OpenVMS.
    OpenVms,

    // Hewlett-Packard Non-Stop Kernel.
    Nsk,

    // AROS.
    Aros,

    // FenixOS.
    FenixOs,

    // Nuxi CloudABI.
    CloudAbi,

    // ARM EABI.
    ArmAeabi,

    // ARM.
    Arm,

    // Standalone (embedded) application.
    Standalone,
}

impl OsAbi {
    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, os_abi) = take(1usize)(input)?;

        Ok((
            input,
            match os_abi[0] {
                0 => Self::SystemV,
                1 => Self::HpUx,
                2 => Self::NetBsd,
                3 => Self::Gnu,
                4 => Self::GnuHurd,
                6 => Self::Solaris,
                7 => Self::Aix,
                8 => Self::Irix,
                9 => Self::FreeBsd,
                10 => Self::Tru64,
                11 => Self::Modesto,
                12 => Self::OpenBsd,
                13 => Self::OpenVms,
                14 => Self::Nsk,
                15 => Self::Aros,
                16 => Self::FenixOs,
                17 => Self::CloudAbi,
                64 => Self::ArmAeabi,
                97 => Self::Arm,
                255 => Self::Standalone,
                _ => Self::None,
            },
        ))
    }
}

#[derive(Debug)]
pub struct File {
    pub endianness: Endianness,
    pub os_abi: OsAbi,
}

impl File {
    const MAGIC: &'static [u8; 4] = &[0x7f, b'E', b'L', b'F'];

    pub fn parse<'a, E>(input: Input<'a>) -> Result<Self, E>
    where
        E: ParseError<Input<'a>>,
    {
        let (input, (_magic, _class, endianness, _version, os_abi)) = tuple((
            tag(Self::MAGIC),
            tag(&[0x2] /* 64 bits */),
            Endianness::parse,
            tag(&[0x1]),
            OsAbi::parse,
        ))(input)?;

        let file = Self { endianness, os_abi };

        Ok((input, file))
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    const EXIT_FILE: &'static [u8] = include_bytes!("../../tests/fixtures/exit.o");

    #[test]
    fn test_me() {
        dbg!(&EXIT_FILE);
        let file = File::parse::<VerboseError<Input>>(EXIT_FILE);
        dbg!(&file);

        assert!(file.is_ok());
    }
}
