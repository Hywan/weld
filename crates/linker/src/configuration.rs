use std::path::PathBuf;

use crate::{target::Triple, Linker};

/// Configuration of the linker.
///
/// This type works like a builder for [`Linker`].
#[derive(Debug)]
pub struct Configuration {
    pub(crate) target: Triple,
    pub(crate) input_files: Vec<PathBuf>,
    pub(crate) _output_file: PathBuf,
}

impl Configuration {
    /// Create a new `Configuration`.
    pub fn new(target: Triple, input_files: Vec<PathBuf>, output_file: PathBuf) -> Self {
        Self { target, input_files, _output_file: output_file }
    }

    /// End the configuration step, and build a [`Linker`].
    pub fn linker(self) -> Linker {
        Linker::with_configuration(self)
    }
}
