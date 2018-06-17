use zlib;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug)]
pub enum ObjError {
    IoError(io::Error),
    HashPrefixTooShort,
    ObjectNotFound,
}

pub struct Object {
    obj_type: String,
    obj_size: usize,
    data: String,
}

pub fn parse(hash_prefix: &str) -> Result<Object, ObjError> {
    let path = object_path(hash_prefix)?;
    let data = fs::read(path).map_err(ObjError::IoError)?;
    let decompressed_data = zlib::decompress(data);

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

fn object_path(hash_prefix: &str) -> Result<PathBuf, ObjError> {
    if hash_prefix.len() < 2 {
        return Err(ObjError::HashPrefixTooShort);
    }

    let dir = Path::new(".git").join("objects").join(&hash_prefix[..2]);
    let filename = &hash_prefix[2..];
    for file in fs::read_dir(dir).map_err(ObjError::IoError)? {
        let path = file.map_err(ObjError::IoError)?.path();
        if path.starts_with(filename) {
            return Ok(path);
        }
    }

    return Err(ObjError::ObjectNotFound);
}
