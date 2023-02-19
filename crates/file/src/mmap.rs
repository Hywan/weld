//! Memmory map file reader.

use std::{
    ffi::c_void,
    fs,
    io::{Error, ErrorKind},
    marker::PhantomData,
    os::fd::AsRawFd,
    ptr, slice,
};

use super::*;

/// File reader based on `mmap(2)`.
pub struct Mmap<'f> {
    _file: fs::File,
    pointer: *const c_void,
    length: usize,
    _phantom: PhantomData<&'f ()>,
}

impl<'f> FileReader for Mmap<'f> {
    type Bytes = &'f [u8];

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

        Ok(Self { _file: file, pointer, length, _phantom: PhantomData })
    }

    fn read_as_bytes(&mut self) -> Result<Self::Bytes> {
        Ok(unsafe { slice::from_raw_parts(self.pointer as *const u8, self.length) })
    }
}

impl<'f> Drop for Mmap<'f> {
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

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as _ }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap() {
        let mut file = Mmap::open("tests/hello.txt").unwrap();
        let content = file.read_as_bytes().unwrap();

        assert_eq!(content, &b"abcdef"[..]);
    }
}
