// * Git Internals - Git Objects
//   https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

use std::fs;
use std::io;
use std::num;
use std::path::{Path, PathBuf};
use std::str;

use zlib;

pub struct Object {
    pub obj_type: String,
    pub obj_size: usize,
    pub data: String,
}

#[derive(Debug)]
pub enum Error {
    HashPrefixTooShort,
    HeaderMissingNullByte,
    HeaderMissingSize,
    HeaderMissingType,
    IoError(io::Error),
    ObjectNotFound,
    ParsingError(num::ParseIntError),
    Utf8Error(str::Utf8Error),
}

pub fn parse(hash_prefix: &str) -> Result<Object, Error> {
    let path = object_path(hash_prefix)?;
    let raw_data = fs::read(path).map_err(Error::IoError)?;
    let decompressed_data = zlib::decompress(raw_data);

    let data = str::from_utf8(&decompressed_data).map_err(Error::Utf8Error)?;

    let header_idx = match data.find('\x00') {
        Some(i) => i,
        None => {
            return Err(Error::HeaderMissingNullByte);
        }
    };
    let (header, data) = data.split_at(header_idx);

    let mut iter = header.split_whitespace();
    let obj_type = match iter.next() {
        Some(tp) => tp.to_string(),
        None => {
            return Err(Error::HeaderMissingType);
        }
    };
    let obj_size = match iter.next() {
        Some(sz) => sz.parse::<usize>().map_err(Error::ParsingError)?,
        None => {
            return Err(Error::HeaderMissingSize);
        }
    };
    let data = data[1..].to_string();

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

    let dir = Path::new(".git").join("objects").join(&hash_prefix[..2]);
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
