//! Default file reader.

use std::{
    fs,
    future::{ready, Ready},
    io::Read,
};

use super::*;

/// File reader based on the standard and default file system calls.
pub struct File {
    inner: fs::File,
}

impl FileReader for File {
    type Bytes = Vec<u8>;
    type Reader = Ready<Result<Self::Bytes>>;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self { inner: fs::File::open(path)? })
    }

    fn read_as_bytes(mut self) -> Self::Reader {
        let mut buffer = Vec::new();

        if let Err(err) = self.inner.read_to_end(&mut buffer) {
            ready(Err(err))
        } else {
            ready(Ok(buffer))
        }
    }
}

#[cfg(test)]
mod tests {
    use smol::block_on;

    use super::*;

    #[test]
    fn test_file() -> Result<()> {
        block_on(async {
            let file = File::open("tests/hello.txt")?;
            let content = file.read_as_bytes().await?;
            let bytes: &[u8] = content.as_ref();

            assert_eq!(bytes, &b"abcdef"[..]);

            Ok(())
        })
    }
}
