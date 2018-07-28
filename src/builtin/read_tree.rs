use std::num;
use std::str;

use object;
use sha1;

#[derive(Debug)]
pub struct Entry {
    pub mode: u32,
    pub path: String,
    pub hash: String,
}

#[derive(Debug)]
pub enum Error {
    ObjectError(object::Error),
    ParseIntError(num::ParseIntError),
    TreeEntryInvalidHash,
    TreeEntryMissingHash,
    TreeEntryMissingMode,
    TreeEntryMissingPath,
    Utf8Error(str::Utf8Error),
}

pub fn read_tree(hash_prefix: &str) -> Result<Vec<Entry>, Error> {
    let mut tree = Vec::new();
    let object = object::get_object(hash_prefix).map_err(Error::ObjectError)?;
    if object.obj_type != "tree" {
        println!("read-tree: object is not a tree but '{}'", object.obj_type);
        return Ok(tree);
    }

    let mut start = 0;
    while start < object.data.len() {
        let end = match object.data[start..].iter().position(|&x| x == 0) {
            Some(i) => i + 21,
            None => break,
        };
        let entry = &object.data[start..end];

        let space_byte = match entry.iter().position(|&x| x == 32) {
            Some(i) => i,
            None => {
                return Err(Error::TreeEntryMissingMode);
            }
        };
        let (mode, entry) = entry.split_at(space_byte);
        let entry = &entry[1..];

        let null_byte = match entry.iter().position(|&x| x == 0) {
            Some(i) => i,
            None => {
                return Err(Error::TreeEntryMissingPath);
            }
        };
        let (path, entry) = entry.split_at(null_byte);

        if entry.len() < 21 {
            return Err(Error::TreeEntryMissingHash);
        }
        let hash = &entry[1..21];

        let mode_str = str::from_utf8(&mode).map_err(Error::Utf8Error)?;
        let mode = u32::from_str_radix(&mode_str, 8).map_err(Error::ParseIntError)?;
        let path = str::from_utf8(&path).map_err(Error::Utf8Error)?.to_string();
        let hash = match sha1::decompress_hash(&hash) {
            Some(hash) => hash,
            None => {
                return Err(Error::TreeEntryInvalidHash);
            }
        };

        tree.push(Entry {
            mode: mode,
            path: path,
            hash: hash.to_string(),
        });
        start = end + 1;
    }

    Ok(tree)
}
