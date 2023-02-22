use std::path::PathBuf;

use crate::Linker;

#[derive(Debug)]
pub struct Configuration {
    pub(crate) input_files: Vec<PathBuf>,
    pub(crate) output_file: PathBuf,
}

impl Configuration {
    pub fn new(input_files: Vec<PathBuf>, output_file: PathBuf) -> Self {
        Self { input_files, output_file }
    }

    pub fn linker(self) -> Linker {
        Linker::with_configuration(self)
    }
}
