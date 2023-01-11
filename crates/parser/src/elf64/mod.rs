use crate::{generators::*, Input, Result};
use weld_parser_macros::EnumParse;

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Endianness {
    Little = 0x1,
    Big = 0x2,
}

#[derive(EnumParse, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OsAbi {
    // [System V](https://en.wikipedia.org/wiki/System_V).
    SystemV = 0x00,

    // [HP-UX](https://en.wikipedia.org/wiki/HP-UX).
    HpUx = 0x01,

    // [NetBSD](https://en.wikipedia.org/wiki/NetBSD).
    NetBsd = 0x02,

    // [GNU Linux](https://en.wikipedia.org/wiki/Linux).
    Gnu = 0x03,

    // [GNU Hurd](https://en.wikipedia.org/wiki/GNU_Hurd).
    GnuHurd = 0x04,

    // [Sun Solaris](https://en.wikipedia.org/wiki/Solaris_(operating_system)).
    Solaris = 0x06,

    // [IBM AIX (Monterey)](https://en.wikipedia.org/wiki/IBM_AIX).
    Aix = 0x07,

    // [SGI IRIX](https://en.wikipedia.org/wiki/IRIX).
    Irix = 0x08,

    // [FreeBSD](https://en.wikipedia.org/wiki/FreeBSD).
    FreeBsd = 0x09,

    // [Compaq TRU64 UNIX](https://en.wikipedia.org/wiki/Tru64).
    Tru64 = 0x0a,

    // Novell Modesto.
    Modesto = 0x0b,

    // [OpenBSD](https://en.wikipedia.org/wiki/OpenBSD).
    OpenBsd = 0x0c,

    // [OpenVMS](https://en.wikipedia.org/wiki/OpenVMS).
    OpenVms = 0x0d,

    // [Hewlett-Packard Non-Stop Kernel](https://en.wikipedia.org/wiki/NonStop_(server_computers)).
    Nsk = 0x0e,

    // [AROS](https://en.wikipedia.org/wiki/AROS_Research_Operating_System).
    Aros = 0x0f,

    // FenixOS.
    FenixOs = 0x10,

    // [Nuxi CloudABI](https://en.wikipedia.org/wiki/CloudABI).
    CloudAbi = 0x11,

    // [Stratus Technologies OpenVOS](https://en.wikipedia.org/wiki/Stratus_VOS).
    OpenVos = 0x12,

    // ARM EABI.
    ArmAeabi = 0x40,

    // ARM.
    Arm = 0x61,

    // Standalone (embedded) application.
    Standalone = 0xff,
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
