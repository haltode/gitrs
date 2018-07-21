use std::io;

use hash_object;
use index;
use utils::sha1;

#[derive(Debug)]
pub enum Error {
    HashError(io::Error),
    IndexError(index::Error),
}

// TODO: recursive walk
pub fn write_tree() -> Result<String, Error> {
    let mut tree = Vec::new();
    let entries = index::read_entries().map_err(Error::IndexError)?;
    for entry in entries {
        let tree_entry = format!("{:o} {}\x00", entry.mode, entry.path);
        let compressed_hash = match sha1::hex_str_to_u8(&entry.hash) {
            Some(hash) => hash,
            None => continue,
        };
        tree.extend(tree_entry.as_bytes());
        tree.extend(compressed_hash);
    }
    let write = true;
    let hash = hash_object::hash_object(&tree, "tree", write).map_err(Error::HashError)?;
    Ok(hash)
}
