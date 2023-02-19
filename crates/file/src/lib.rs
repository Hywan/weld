use std::{io::Result, path::Path};

#[cfg(all(not(feature = "auto"), not(feature = "fs"), not(feature = "mmap")))]
compile_error!("No feature has been selected, please select at least `auto`");

#[cfg(feature = "mmap")]
pub mod mmap;

#[cfg(feature = "fs")]
pub mod fs;

pub trait FileReader: Sized {
    type Bytes: AsRef<[u8]>;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>;

    fn read_as_bytes(&mut self) -> Result<Self::Bytes>;
}

/// File picker.
///
/// This type opens a file path based on the file reader selected by a Cargo
/// feature (e.g. `fs` or `mmap`).
pub struct Picker;

impl Picker {
    #[cfg(feature = "fs")]
    pub fn open<P>(path: P) -> Result<fs::File>
    where
        P: AsRef<Path>,
    {
        fs::File::open(path)
    }

    #[cfg(feature = "mmap")]
    pub fn open<'f, P>(path: P) -> Result<mmap::Mmap<'f>>
    where
        P: AsRef<Path>,
    {
        mmap::Mmap::open(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker() {
        let mut file = Picker::open("tests/hello.txt").unwrap();
        let content = file.read_as_bytes().unwrap();

        assert_eq!(content, &b"abcdef"[..]);
    }
}
