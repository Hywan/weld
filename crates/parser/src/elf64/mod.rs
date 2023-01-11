use crate::{generators::*, Input, Result};

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

    // [System V](https://en.wikipedia.org/wiki/System_V).
    SystemV,

    // [HP-UX](https://en.wikipedia.org/wiki/HP-UX).
    HpUx,

    // [NetBSD](https://en.wikipedia.org/wiki/NetBSD).
    NetBsd,

    // [GNU Linux](https://en.wikipedia.org/wiki/Linux).
    Gnu,

    // [GNU Hurd](https://en.wikipedia.org/wiki/GNU_Hurd).
    GnuHurd,

    // [Sun Solaris](https://en.wikipedia.org/wiki/Solaris_(operating_system)).
    Solaris,

    // [IBM AIX (Monterey)](https://en.wikipedia.org/wiki/IBM_AIX).
    Aix,

    // [SGI IRIX](https://en.wikipedia.org/wiki/IRIX).
    Irix,

    // [FreeBSD](https://en.wikipedia.org/wiki/FreeBSD).
    FreeBsd,

    // [Compaq TRU64 UNIX](https://en.wikipedia.org/wiki/Tru64).
    Tru64,

    // Novell Modesto.
    Modesto,

    // [OpenBSD](https://en.wikipedia.org/wiki/OpenBSD).
    OpenBsd,

    // [OpenVMS](https://en.wikipedia.org/wiki/OpenVMS).
    OpenVms,

    // [Hewlett-Packard Non-Stop Kernel](https://en.wikipedia.org/wiki/NonStop_(server_computers)).
    Nsk,

    // [AROS](https://en.wikipedia.org/wiki/AROS_Research_Operating_System).
    Aros,

    // FenixOS.
    FenixOs,

    // [Nuxi CloudABI](https://en.wikipedia.org/wiki/CloudABI).
    CloudAbi,

    // [Stratus Technologies OpenVOS](https://en.wikipedia.org/wiki/Stratus_VOS).
    OpenVos,

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
                0x00 => Self::SystemV,
                0x01 => Self::HpUx,
                0x02 => Self::NetBsd,
                0x03 => Self::Gnu,
                0x04 => Self::GnuHurd,
                0x06 => Self::Solaris,
                0x07 => Self::Aix,
                0x08 => Self::Irix,
                0x09 => Self::FreeBsd,
                0x0a => Self::Tru64,
                0x0b => Self::Modesto,
                0x0c => Self::OpenBsd,
                0x0d => Self::OpenVms,
                0x0e => Self::Nsk,
                0x0f => Self::Aros,
                0x10 => Self::FenixOs,
                0x11 => Self::CloudAbi,
                0x12 => Self::OpenVos,
                0x40 => Self::ArmAeabi,
                0x61 => Self::Arm,
                0xff => Self::Standalone,
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
        let (input, (_magic, _class, endianness, _version, os_abi, _padding)) = tuple((
            tag(Self::MAGIC),
            tag(&[0x2] /* 64 bits */),
            Endianness::parse,
            tag(&[0x1]),
            OsAbi::parse,
            skip(8usize),
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
        let file = File::parse::<VerboseError<Input>>(EXIT_FILE);
        dbg!(&file);

        assert!(file.is_ok());
    }
}
