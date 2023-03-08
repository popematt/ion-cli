use std::io::{self, Read, Write};
use std::path::Path;
use std::rc::Rc;

pub use real_fs::RealFileSystem;
pub use fake_fs::FakeFileSystem;

pub trait ReadWrite: Read + Write {}
impl <T> ReadWrite for T where T: Read, T: Write {}
pub type FileWrapper<T: ReadWrite> = T;

pub trait FileSystemWrapper<'a> {
    type FileType: ReadWrite;
    fn create(&'a mut self, path: &dyn AsRef<Path>) -> io::Result<Self::FileType>;
    fn open(&'a mut self, path: &dyn AsRef<Path>) -> io::Result<Self::FileType>;
    fn tempfile(&'a mut self) -> io::Result<Self::FileType>;
}

mod fake_file {
    use std::cell::{Ref, RefCell, RefMut};
    use std::cmp;
    use std::io::Cursor;
    use super::*;

    pub struct FakeFileData(Rc<RefCell<Vec<u8>>>);
    impl FakeFileData {
        pub fn new() -> Self {
            FakeFileData(Rc::new(RefCell::new(vec![])))
        }
        pub fn as_fake_file<'a>(&'a self) -> FakeFile<'a> {
            FakeFile::Borrowed(BorrowedFakeFile {
                inner: RefCell::borrow_mut(&self.0),
                pos: 0,
            })
        }
    }

    pub struct BorrowedFakeFile<'a> {
        inner: RefMut<'a, Vec<u8>>,
        pos: usize,
    }
    impl BorrowedFakeFile<'_> {
        pub fn remaining_slice(&self) -> &[u8] {
            let len = self.pos.min(<Vec<u8> as AsRef<Vec<u8>>>::as_ref(&self.inner).len());
            &self.inner[(len as usize)..]
        }
    }
    impl <'a> Write for BorrowedFakeFile<'a> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let pos = cmp::min(self.pos, self.inner.len());
            let amt = (&mut self.inner[pos..]).write(buf)?;
            self.pos += amt;
            Ok(amt)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl <'a> Read for BorrowedFakeFile<'a> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = Read::read(&mut self.remaining_slice(), buf)?;
            self.pos += n;
            Ok(n)
        }
    }

    pub enum FakeFile<'a> {
        Owned(Cursor<Vec<u8>>),
        Borrowed(BorrowedFakeFile<'a>)
    }
    impl <'a> Write for FakeFile<'a> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self {
                FakeFile::Owned(cursor) => cursor.write(buf),
                FakeFile::Borrowed(fake_file) => fake_file.write(buf),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self {
                FakeFile::Owned(cursor) => cursor.flush(),
                FakeFile::Borrowed(fake_file) => fake_file.flush(),
            }
        }
    }
    impl <'a> Read for FakeFile<'a> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            match self {
                FakeFile::Owned(cursor) => cursor.read(buf),
                FakeFile::Borrowed(fake_file) => fake_file.read(buf),
            }
        }
    }
}
mod fake_fs {
    use super::{*, fake_file::FakeFileData};
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::marker::PhantomData;
    use crate::fs_wrapper::fake_file::FakeFile;

    pub struct FakeFileSystem<'a> {
        files: HashMap<String, FakeFileData>,
        _lifetime: PhantomData<&'a ()>,
    }
    impl <'a> FakeFileSystem<'a> {
        pub fn new() -> Self {
            FakeFileSystem { files: HashMap::new(), _lifetime: PhantomData::default() }
        }
    }

    impl <'a> FileSystemWrapper<'a> for FakeFileSystem<'a> {
        type FileType = FakeFile<'a>;
        fn create(&'a mut self, path: &dyn AsRef<Path>) -> io::Result<FakeFile<'a>> {
            let binding = path.as_ref().to_path_buf();
            let key = binding.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;
            let map = &mut self.files;
            if map.contains_key(key) {
                Err(io::Error::new(io::ErrorKind::AlreadyExists, "File already exists"))
            } else {
                let entry = map.entry(key.to_string()).or_insert(FakeFileData::new());
                Ok(entry.as_fake_file())
            }
        }

        fn open(&'a mut self, path: &dyn AsRef<Path>) -> io::Result<FakeFile<'a>> {
            let binding = path.as_ref().to_path_buf();
            let key = binding.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;
            let map = &mut self.files;
            match map.get(key).as_mut() {
                None => Err(io::Error::new(io::ErrorKind::NotFound, "File does not exist")),
                Some(it) => {
                    Ok(it.as_fake_file())
                }
            }
        }
        fn tempfile(&'a mut self) -> io::Result<FakeFile<'a>> {
            Ok(FakeFile::Owned(Cursor::new(vec![])))
        }
    }
}

mod real_fs {
    use super::*;
    use std::fs::File;

    pub struct RealFileSystem;
    const REAL_FS: RealFileSystem = RealFileSystem;

    impl RealFileSystem {
        pub fn new() -> Self {
            REAL_FS
        }
    }
    impl FileSystemWrapper<'static> for RealFileSystem {
        type FileType = File;
        fn create(&mut self, path: &dyn AsRef<Path>) -> io::Result<File> {
            File::create(path)
        }
        fn open(&mut self, path: &dyn AsRef<Path>) -> io::Result<File> {
            File::open(path)
        }
        fn tempfile(&mut self) -> io::Result<File> {
            tempfile::tempfile()
        }
    }
}