use thiserror::Error;

#[allow(unused)]
use crate::target::BinaryFormat;
use crate::{target::Triple, Configuration};

#[derive(Debug)]
pub struct Linker {
    configuration: Configuration,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("no input file given")]
    NoInputFile,

    #[error("unsupported target triple `{0:?}`")]
    UnsupportedTarget(Triple),

    #[cfg(feature = "elf64")]
    #[error("elf64 error: {0}")]
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

            _ => return Err(Error::UnsupportedTarget(self.configuration.target)),
        })
    }
}
