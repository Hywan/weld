use std::{io::Result, path::Path};

pub trait FileReader: Sized {
    type Bytes: AsRef<[u8]>;

    fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>;

    fn create<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>;

    fn read_as_bytes(&mut self) -> Result<Self::Bytes>;
}

mod file {
    use std::{fs, io::Read};

    use super::*;

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

        fn create<P>(path: P) -> Result<Self>
        where
            P: AsRef<Path>,
        {
            Ok(Self { inner: fs::File::create(path)? })
        }

        fn read_as_bytes(&mut self) -> Result<Self::Bytes> {
            let mut buffer = Vec::new();
            self.inner.read_to_end(&mut buffer)?;

            Ok(buffer)
        }
    }
}

mod mmap {
    use std::{
        ffi::c_void,
        fs,
        io::{Error, ErrorKind},
        marker::PhantomData,
        os::fd::AsRawFd,
        ptr, slice,
    };

    use super::*;

    pub struct Mmap<'f> {
        file: fs::File,
        pointer: *mut c_void,
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
                let length = file.metadata()?.len();

                if length > (usize::MAX as u64) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "memory map length overflows usize",
                    ));
                }

                if length == 0 {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "memory map must have a non-zero length",
                    ));
                }

                length as usize
            };

            let protections = libc::PROT_READ | libc::PROT_EXEC;
            let flags = libc::MAP_SHARED;
            let alignment = 0;

            let pointer = unsafe {
                let pointer = libc::mmap(
                    ptr::null_mut(),
                    length as libc::size_t,
                    protections,
                    flags,
                    file.as_raw_fd(),
                    0 as libc::off_t,
                );

                if pointer == libc::MAP_FAILED {
                    return Err(Error::last_os_error());
                }

                pointer
            };

            Ok(Self { file, pointer, length, _phantom: PhantomData })
        }

        fn create<P>(path: P) -> Result<Self>
        where
            P: AsRef<Path>,
        {
            todo!()
        }

        fn read_as_bytes(&mut self) -> Result<Self::Bytes> {
            Ok(unsafe { slice::from_raw_parts(self.pointer as *mut u8, self.length) })
        }
    }

    impl<'f> Drop for Mmap<'f> {
        fn drop(&mut self) {
            // TODO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file() {
        let mut file = file::File::open("tests/hello.txt").unwrap();
        let content = file.read_as_bytes().unwrap();

        assert_eq!(content, &b"abcdef"[..]);
    }

    #[test]
    fn test_mmap() {
        let mut file = mmap::Mmap::open("tests/hello.txt").unwrap();
        let content = file.read_as_bytes().unwrap();

        assert_eq!(content, &b"abcdef"[..]);
    }
}
