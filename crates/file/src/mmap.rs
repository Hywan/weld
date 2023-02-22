//! Memmory map file reader.

use std::{
    ffi::c_void,
    fs,
    future::{ready, Ready},
    io::{Error, ErrorKind},
    os::fd::AsRawFd,
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
            let pointer = libc::mmap(
                ptr::null_mut(),
                length as libc::size_t,
                libc::PROT_READ,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0 as libc::off_t,
            );

            if pointer == libc::MAP_FAILED {
                return Err(Error::last_os_error());
            }

            pointer
        };

        Ok(Self { content: MmapContent { _file: file, pointer, length } })
    }

    fn read_as_bytes(self) -> Self::Reader {
        ready(Ok(self.content))
    }
}

pub struct MmapContent {
    _file: fs::File,
    pointer: *const c_void,
    length: usize,
}

impl AsRef<[u8]> for MmapContent {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.pointer as *const u8, self.length) }
    }
}

impl Drop for MmapContent {
    fn drop(&mut self) {
        let alignment = self.pointer as usize % page_size();

        assert_eq!(
            unsafe {
                libc::munmap(
                    self.pointer.offset(-(alignment as isize)) as *mut _,
                    self.length as libc::size_t,
                )
            },
            0,
            "Failed to unmap the memory map: {}",
            Error::last_os_error()
        );
    }
}

// SAFETY: `MmapContent.pointer`'s lifetime is tied to `MmapContent.file`.
unsafe impl Send for MmapContent {}

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as _ }
}

#[cfg(test)]
mod tests {
    use smol::block_on;

    use super::*;

    #[test]
    fn test_mmap() -> Result<()> {
        block_on(async {
            let file = Mmap::open("tests/hello.txt")?;
            let content = file.read_as_bytes().await?;
            let bytes: &[u8] = content.as_ref();

            assert_eq!(bytes, &b"abcdef"[..]);

            Ok(())
        })
    }
}
