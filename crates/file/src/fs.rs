//! Default file reader.

use std::{fs, io::Read};

use super::*;

/// File reader based on the standard and default file system calls.
pub struct File {
    inner: fs::File,
}

impl FileReader for File {
    type Bytes = Vec<u8>;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self { inner: fs::File::open(path)? })
    }

    fn read_as_bytes(&mut self) -> Result<Self::Bytes> {
        let mut buffer = Vec::new();
        self.inner.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file() {
        let mut file = File::open("tests/hello.txt").unwrap();
        let content = file.read_as_bytes().unwrap();

        assert_eq!(content, &b"abcdef"[..]);
    }
}
