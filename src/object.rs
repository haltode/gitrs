// * Git Internals - Git Objects
//   https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

use std::fs;
use std::io;
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
    IoError(io::Error),
    HashPrefixTooShort,
    ObjectNotFound,
}

pub fn parse(hash_prefix: &str) -> Result<Object, Error> {
    let path = object_path(hash_prefix)?;
    let raw_data = fs::read(path).map_err(Error::IoError)?;
    let decompressed_data = zlib::decompress(raw_data);

    let data = str::from_utf8(&decompressed_data).expect("invalid utf-8 in object data");

    let header_idx = data.find('\x00')
        .expect("missing null byte in object header");
    let (header, data) = data.split_at(header_idx);

    let mut iter = header.split_whitespace();
    let obj_type = iter.next()
        .expect("missing type in object header")
        .to_string();
    let obj_size = iter.next()
        .expect("missing size in object header")
        .parse::<usize>()
        .expect("cannot convert size in object header");
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
        if path.file_name().expect("error in file name") == filename {
            return Ok(path);
        }
    }

    Err(Error::ObjectNotFound)
}
