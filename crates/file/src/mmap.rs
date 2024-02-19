//! Memmory map file reader.

use rustix::{
    mm::{mmap, munmap, MapFlags, ProtFlags},
    param::page_size,
};
use std::{
    ffi::c_void,
    fs,
    future::{ready, Ready},
    io::{Error, ErrorKind},
    ptr, slice,
};

use super::*;

/// File reader based on `mmap(2)`.
pub struct Mmap {
    content: MmapContent,
}

impl FileReader for Mmap {
    type Bytes = MmapContent;
    type Reader = Ready<Result<Self::Bytes>>;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;

        let length = {
            let length: usize = file.metadata()?.len().try_into().map_err(|_| {
                Error::new(
                    ErrorKind::InvalidData,
                    "Memory map length is too large to fit in `usize`",
                )
            })?;

            if length == 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Memory map's length must not be zero",
                ));
            }

            length
        };

        let pointer = unsafe {
            mmap(ptr::null_mut(), length, ProtFlags::READ, MapFlags::SHARED, &file, 0)
                .map_err(|errno| Error::from_raw_os_error(errno.raw_os_error()))?
        };

        Ok(Self { content: MmapContent { _file: file, pointer, length } })
    }

    fn read_as_bytes(self) -> Self::Reader {
        ready(Ok(self.content))
    }
}

/// Represents the content read from a [`Mmap`].
pub struct MmapContent {
    _file: fs::File,
    pointer: *const c_void,
    length: usize,
}

impl Deref for MmapContent {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.pointer as *const u8, self.length) }
    }
}

impl Drop for MmapContent {
    fn drop(&mut self) {
        let alignment = self.pointer as usize % page_size();

        unsafe { munmap(self.pointer.offset(-(alignment as isize)) as *mut _, self.length) }
            .map_err(|errno| Error::from_raw_os_error(errno.raw_os_error()))
            .unwrap();
    }
}

// SAFETY: `MmapContent.pointer`'s lifetime is tied to `MmapContent.file`.
unsafe impl Send for MmapContent {}

#[cfg(test)]
mod tests {
    use futures_lite::future::block_on;

    use super::*;

    #[test]
    fn test_mmap() -> Result<()> {
        block_on(async {
            let file = Mmap::open("tests/hello.txt")?;
            let content = file.read_as_bytes().await?;

            assert_eq!(*content, b"abcdef"[..]);

            Ok(())
        })
    }
}
