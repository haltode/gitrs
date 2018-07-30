// * Git Internals - Git Objects
//   https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;

use bits::big_endian;
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
    InvalidObject,
    IoError(io::Error),
    ObjectNotFound,
    Utf8Error(str::Utf8Error),
}

pub fn get_object(hash_prefix: &str) -> Result<Object, Error> {
    let path = object_path(hash_prefix)?;
    let raw_data = fs::read(path).map_err(Error::IoError)?;
    let data = zlib::decompress(raw_data);

    let header_idx = match data.iter().position(|&x| x == 0) {
        Some(i) => i,
        None => return Err(Error::HeaderMissingNullByte),
    };
    let (header, data) = data.split_at(header_idx);

    // 32 = space character (ASCII)
    let mut iter = header.split(|&x| x == 32);
    let obj_type = match iter.next() {
        Some(tp) => str::from_utf8(&tp).map_err(Error::Utf8Error)?.to_string(),
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

pub fn get_tree_from_commit(hash: &str) -> Result<String, Error> {
    let object = get_object(&hash)?;
    if object.data.len() < 45 {
        return Err(Error::InvalidObject);
    }
    let tree: String = object.data[5..45].iter().map(|&x| x as char).collect();
    Ok(tree)
}

fn object_path(hash_prefix: &str) -> Result<PathBuf, Error> {
    if hash_prefix.len() < 2 {
        return Err(Error::HashPrefixTooShort);
    }

    let (dir, file) = hash_prefix.split_at(2);
    let objects = Path::new(".git").join("objects").join(dir);
    for f in fs::read_dir(objects).map_err(Error::IoError)? {
        let path = f.map_err(Error::IoError)?.path();
        if let Some(f) = path.file_name() {
            if f == file {
                return Ok(path.clone());
            }
        }
    }

    Err(Error::ObjectNotFound)
}
