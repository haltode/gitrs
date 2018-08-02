use std::str;

use builtin::cat_file;
use object;
use object::Object;
use sha1;

#[derive(Debug)]
pub struct Entry {
    pub mode: u32,
    pub path: String,
    pub hash: String,
}

#[derive(Debug)]
pub enum Error {
    NotATreeObject,
    ObjectError(object::Error),
    TreeEntryInvalidHash,
    TreeEntryMissingHash,
    TreeEntryMissingMode,
    TreeEntryMissingPath,
}

pub fn cmd_read_tree(args: &[String]) {
    if args.is_empty() {
        println!("read-tree: command takes a 'hash' argument.");
    } else {
        let hash = &args[0];
        if let Err(why) = cat_file::cat_file(hash, "--print") {
            println!("Cannot retrieve object info: {:?}", why);
        }
    }
}

pub fn read_tree(hash_prefix: &str) -> Result<Vec<Entry>, Error> {
    let object = Object::new(&hash_prefix).map_err(Error::ObjectError)?;
    if object.obj_type != "tree" {
        return Err(Error::NotATreeObject);
    }

    let mut tree = Vec::new();
    let mut start = 0;
    while start < object.data.len() {
        let end = match object.data[start..].iter().position(|&x| x == 0) {
            Some(i) => start + i + 21,
            None => break,
        };
        let entry = &object.data[start..end];

        let space_byte = match entry.iter().position(|&x| x == 32) {
            Some(i) => i,
            None => return Err(Error::TreeEntryMissingMode),
        };
        let (mode, entry) = entry.split_at(space_byte);
        let entry = &entry[1..];

        let null_byte = match entry.iter().position(|&x| x == 0) {
            Some(i) => i,
            None => return Err(Error::TreeEntryMissingPath),
        };
        let (path, entry) = entry.split_at(null_byte);

        if entry.len() < 21 {
            return Err(Error::TreeEntryMissingHash);
        }
        let hash = &entry[1..21];

        let mode_str = str::from_utf8(&mode).unwrap();
        let mode = u32::from_str_radix(&mode_str, 8).unwrap();
        let path = str::from_utf8(&path).unwrap();
        let hash = match sha1::decompress_hash(&hash) {
            Some(hash) => hash,
            None => return Err(Error::TreeEntryInvalidHash),
        };

        tree.push(Entry {
            mode: mode,
            path: path.to_string(),
            hash: hash.to_string(),
        });
        start = end;
    }

    Ok(tree)
}
