// * Git Internals - Git Objects
//   https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

use std::fs;
use std::io;
use std::path::PathBuf;
use std::str;

use bits::big_endian;
use environment;
use zlib;

pub struct Object {
    pub obj_type: String,
    pub obj_size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    HashPrefixTooShort,
    HeaderMissingNullByte,
    HeaderMissingSize,
    HeaderMissingType,
    IoError(io::Error),
    ObjectNotFound,
    Utf8Error(str::Utf8Error),
    WorkingDirError(environment::Error),
}

pub fn get_object(hash_prefix: &str) -> Result<Object, Error> {
    let path = object_path(hash_prefix)?;
    let raw_data = fs::read(path).map_err(Error::IoError)?;
    let data = zlib::decompress(raw_data);

    let header_idx = match data.iter().position(|&x| x == 0) {
        Some(i) => i,
        None => {
            return Err(Error::HeaderMissingNullByte);
        }
    };
    let (header, data) = data.split_at(header_idx);

    // 32 = space character (ASCII)
    let mut iter = header.split(|&x| x == 32);
    let obj_type = match iter.next() {
        Some(tp) => str::from_utf8(&tp).map_err(Error::Utf8Error)?.to_string(),
        None => {
            return Err(Error::HeaderMissingType);
        }
    };
    let obj_size = match iter.next() {
        Some(sz) => big_endian::u8_slice_to_usize(sz),
        None => {
            return Err(Error::HeaderMissingSize);
        }
    };
    // Skip the null byte
    let data = data[1..].to_vec();

    Ok(Object {
        obj_type,
        obj_size,
        data,
    })
}

fn object_path(hash_prefix: &str) -> Result<PathBuf, Error> {
    if hash_prefix.len() < 2 {
        return Err(Error::HashPrefixTooShort);
    }

    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    let dir = git_dir.join("objects").join(&hash_prefix[..2]);
    let filename = &hash_prefix[2..];
    for file in fs::read_dir(dir).map_err(Error::IoError)? {
        let path = file.map_err(Error::IoError)?.path();
        if let Some(f) = path.file_name() {
            if f == filename {
                return Ok(path.clone());
            }
        }
    }

    Err(Error::ObjectNotFound)
}
