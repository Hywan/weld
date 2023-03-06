//! `weld-linker` contains the linking drivers/strategies to actually link
//! object files together.

#![deny(unused)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]

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
