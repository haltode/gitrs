// * Git Internals - Git Objects
//   https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;

use bits::big_endian;
use zlib;

#[derive(Debug)]
pub enum Error {
    HashPrefixTooShort,
    HeaderMissingNullByte,
    HeaderMissingSize,
    HeaderMissingType,
    IoError(io::Error),
    ObjectNotFound,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

#[derive(Debug)]
pub struct Object {
    pub obj_type: String,
    pub obj_size: usize,
    pub data: Vec<u8>,
}

impl Object {
    pub fn new(hash_prefix: &str) -> Result<Object, Error> {
        let path = Object::full_path(hash_prefix)?;
        let raw_data = fs::read(path)?;
        let data = zlib::decompress(raw_data);

        let header_idx = match data.iter().position(|&x| x == 0) {
            Some(i) => i,
            None => return Err(Error::HeaderMissingNullByte),
        };
        let (header, data) = data.split_at(header_idx);

        // 32 = space character (ASCII)
        let mut iter = header.split(|&x| x == 32);
        let obj_type = match iter.next() {
            Some(tp) => str::from_utf8(&tp).expect("invalid utf-8").to_string(),
            None => return Err(Error::HeaderMissingType),
        };
        let obj_size = match iter.next() {
            Some(sz) => big_endian::u8_slice_to_usize(sz),
            None => return Err(Error::HeaderMissingSize),
        };
        // Skip the null byte
        let data = data[1..].to_vec();

        Ok(Object {
            obj_type,
            obj_size,
            data,
        })
    }

    fn full_path(hash_prefix: &str) -> Result<PathBuf, Error> {
        if hash_prefix.len() < 2 {
            return Err(Error::HashPrefixTooShort);
        }

        let (dir, file) = hash_prefix.split_at(2);
        let objects = Path::new(".git").join("objects").join(dir);
        for f in fs::read_dir(objects)? {
            let path = f?.path();
            if let Some(f) = path.file_name() {
                if f == file {
                    return Ok(path.clone());
                }
            }
        }

        Err(Error::ObjectNotFound)
    }
}
