use weld_errors::{error, Result};

use crate::{
    target::{self, Triple},
    Configuration,
};

/// The linker itself.
///
/// To construct a linker, please use the [`Configuration`] type.
///
/// This type is responsible to select the appropriate linking driver and to run
/// it.
#[derive(Debug)]
pub struct Linker {
    configuration: Configuration,
}

error! {
    #[doc = "Linker errors."]
    pub enum Error {
        #[code = E002]
        #[message = "I'm happy to link objects, but no objects was given."]
        #[help = "Maybe try adding input object files with `weld <input_files> …`."]
        NoInputFile,

        #[code = E003]
        #[message = "I understand the given target triple, but I unfortunately don't support its binary format."]
        #[formatted_message("I understand the `{0}` target triple, but I unfortunately don't support its binary format, `{}`.", .0.binary_format)]
        #[help = "Maybe try another target with `weld --target <target>`?"]
        UnsupportedBinaryFormat(Triple),

        #[cfg(feature = "elf64")]
        #[transparent]
        Elf64(#[from] crate::elf64::Error),
    }
}

impl Linker {
    pub(crate) fn with_configuration(configuration: Configuration) -> Self {
        Self { configuration }
    }

    /// Let's weld things!
    pub fn link(self) -> Result<(), Error> {
        if self.configuration.input_files.is_empty() {
            return Err(Error::NoInputFile);
        }

        Ok(match self.configuration.target.binary_format {
            #[cfg(feature = "elf64")]
            target::BinaryFormat::Elf => crate::elf64::link(self.configuration)?,

            _ => return Err(Error::UnsupportedBinaryFormat(self.configuration.target)),
        })
    }
}
