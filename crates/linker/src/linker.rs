use miette::{Diagnostic, Result};
use thiserror::Error;

#[allow(unused)]
use crate::target::BinaryFormat;
use crate::{target::Triple, Configuration};

#[derive(Debug)]
pub struct Linker {
    configuration: Configuration,
}

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error("I'm happy to link objects, but no objects was given")]
    #[diagnostic(
        code(E002),
        help("Maybe try adding input object files with `weld <input_files> …`")
    )]
    NoInputFile,

    #[error("I understand the `{0}` target triple, but I unfortunately don't support its binary format, `{}`.", .0.binary_format)]
    #[diagnostic(
        code(E003),
        help("Maybe try another target with `weld --target <target>`?"),
        url(docsrs)
    )]
    UnsupportedBinaryFormat(Triple),

    #[cfg(feature = "elf64")]
    #[error("I have received an error from the `elf64` driver:\n{0}")]
    Elf64(#[from] crate::elf64::Error),
}

impl Linker {
    pub(crate) fn with_configuration(configuration: Configuration) -> Self {
        Self { configuration }
    }

    pub fn link(self) -> Result<(), Error> {
        if self.configuration.input_files.is_empty() {
            return Err(Error::NoInputFile);
        }

        Ok(match self.configuration.target.binary_format {
            #[cfg(feature = "elf64")]
            BinaryFormat::Elf => crate::elf64::link(self.configuration)?,

            _ => return Err(Error::UnsupportedBinaryFormat(self.configuration.target)),
        })
    }
}
