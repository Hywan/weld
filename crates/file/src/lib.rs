use std::{future::Future, io::Result, path::Path};

#[cfg(all(not(feature = "auto"), not(feature = "fs"), not(feature = "mmap")))]
compile_error!("No feature has been selected, please select at least `auto`");

#[cfg(feature = "mmap")]
pub mod mmap;

#[cfg(feature = "fs")]
pub mod fs;

/// Define what a file reader should look like.
pub trait FileReader: Sized {
    /// The reader should outputs bytes that implements `AsRef<[u8]>`.
    type Bytes: AsRef<[u8]>;
    /// The reader itself is asynchronous.
    type Reader: Future<Output = Result<Self::Bytes>> + Send;

    /// Open a file.
    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>;

    /// Read the entire file content.
    fn read_as_bytes(self) -> Self::Reader;
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
    pub fn open<P>(path: P) -> Result<mmap::Mmap>
    where
        P: AsRef<Path>,
    {
        mmap::Mmap::open(path)
    }
}

#[cfg(test)]
mod tests {
    use smol::block_on;

    use super::*;

    #[test]
    fn test_picker() -> Result<()> {
        block_on(async {
            let file = Picker::open("tests/hello.txt")?;
            let content = file.read_as_bytes().await?;
            let bytes: &[u8] = content.as_ref();

            assert_eq!(bytes, &b"abcdef"[..]);

            Ok(())
        })
    }
}
