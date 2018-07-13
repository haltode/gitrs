use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use hash_object;
use index;

pub fn add(paths: &[String]) {
    let mut entries = index::get_entries().expect("cannot retrieve index entries");
    for path in paths {
        let already_indexed = entries.iter().any(|e| &e.path == path);
        if already_indexed {
            continue;
        }

        let fpath = Path::new(&path);
        let data = fs::read_to_string(&fpath).expect("cannot read file");
        let meta = fs::metadata(&fpath).expect("cannot get file metadata");

        let write = true;
        let hash = hash_object::hash_object(&data, "blob", write).expect("cannot hash object");
        let flags = path.len() as u16;

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
            flags: flags,
            path: path.to_string(),
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    index::write_entries(entries);
}
