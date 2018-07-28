use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use hash_object;
use index;

#[derive(Debug)]
pub enum Error {
    HashError(hash_object::Error),
    IndexError(index::Error),
    IoError(io::Error),
}

pub fn add(paths: &[String]) -> Result<(), Error> {
    let mut entries = index::read_entries().map_err(Error::IndexError)?;
    entries.retain(|e| !paths.contains(&e.path));

    for path in paths {
        let file = Path::new(&path);
        let data = fs::read(&file).map_err(Error::IoError)?;
        let meta = fs::metadata(&file).map_err(Error::IoError)?;

        let write = true;
        let hash = hash_object::hash_object(&data, "blob", write).map_err(Error::HashError)?;

        entries.push(index::Entry {
            ctime_sec: meta.ctime() as u32,
            ctime_nan: meta.ctime_nsec() as u32,
            mtime_sec: meta.mtime() as u32,
            mtime_nan: meta.mtime_nsec() as u32,
            dev: meta.dev() as u32,
            ino: meta.ino() as u32,
            mode: meta.mode(),
            uid: meta.uid(),
            gid: meta.gid(),
            size: meta.size() as u32,
            hash: hash,
            flags: path.len() as u16,
            path: path.to_string(),
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    index::write_entries(entries).map_err(Error::IndexError)?;

    Ok(())
}