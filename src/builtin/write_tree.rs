use std::io;

use builtin::hash_object;
use index;
use sha1;

#[derive(Debug)]
pub enum Error {
    HashObjError(io::Error),
    IndexError(index::Error),
}

pub fn cmd_write_tree() {
    match write_tree() {
        Ok(hash) => println!("{}", hash),
        Err(why) => println!("Could not create tree object: {:?}", why),
    };
}

pub fn write_tree() -> Result<String, Error> {
    let mut tree = Vec::new();
    let entries = index::read_entries().map_err(Error::IndexError)?;
    for entry in entries {
        let tree_entry = format!("{:o} {}\x00", entry.mode, entry.path);
        let compressed_hash = match sha1::compress_hash(&entry.hash) {
            Some(hash) => hash,
            None => continue,
        };
        tree.extend(tree_entry.as_bytes());
        tree.extend(compressed_hash);
    }

    let write = true;
    let hash = hash_object::hash_object(&tree, "tree", write).map_err(Error::HashObjError)?;
    Ok(hash)
}
