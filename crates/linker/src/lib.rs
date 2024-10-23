//! `weld-linker` contains the linking drivers/strategies to actually link
//! object files together.

mod configuration;
#[cfg(feature = "elf64")]
mod elf64;
mod linker;

pub use configuration::*;
#[cfg(feature = "elf64")]
pub use elf64::Error as Elf64Error;
pub use linker::*;

/// This module contains all types to work with target tiple.
pub mod target {
    pub use target_lexicon::*;
}
