// Resources used:
//  * git/Documentation/technical/index-format.txt
//  https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

use bits::big_endian;
use hash_object;
use sha1;

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;

pub struct Entry {
    ctime_sec: u32,
    ctime_nan: u32,
    mtime_sec: u32,
    mtime_nan: u32,
    dev: u32,
    ino: u32,
    pub mode: u32,
    uid: u32,
    gid: u32,
    size: u32,
    pub hash: String,
    pub flags: u16,
    pub path: String,
}

pub fn get_entries() -> Vec<Entry> {
    let bytes = fs::read(Path::new(".git").join("index")).expect("cannot read index");
    let signature = str::from_utf8(&bytes[0..4]).expect("invalid utf-8 in index signature");
    if signature != "DIRC" {
        panic!("invalid header signature in the index");
    }

    let version = big_endian::u8_slice_to_u32(&bytes[4..]);
    if version != 2 {
        panic!("cannot handle index version other than 2");
    }

    let nb_entries = big_endian::u8_slice_to_u32(&bytes[8..]) as usize;
    let mut entries = Vec::new();
    let mut idx = 12;
    for _ in 0..nb_entries {
        let mut fields = [0u32; 10];
        for e in 0..10 {
            fields[e] = big_endian::u8_slice_to_u32(&bytes[idx..]);
            idx += 4;
        }

        let hash = sha1::hash_from_u8_slice(&bytes[idx..]);
        idx += 20;

        let flags = big_endian::u8_slice_to_u16(&bytes[idx..]);
        idx += 2;

        let null_idx = bytes[idx..]
            .iter()
            .position(|&x| x == 0)
            .expect("index entry does not terminate by null byte");
        let path = str::from_utf8(&bytes[idx..idx + null_idx])
            .expect("invalid utf-8 in entry's path")
            .to_string();
        idx += null_idx;

        let entry_len = 62 + path.len();
        let padding = ((entry_len + 8) / 8) * 8 - entry_len;
        idx += padding;

        entries.push(Entry {
            ctime_sec: fields[0],
            ctime_nan: fields[1],
            mtime_sec: fields[2],
            mtime_nan: fields[3],
            dev: fields[4],
            ino: fields[5],
            mode: fields[6],
            uid: fields[7],
            gid: fields[8],
            size: fields[9],
            hash: hash,
            flags: flags,
            path: path,
        });
    }

    // TODO: checksum

    return entries;
}

pub fn status() -> Result<(), io::Error> {
    // TODO: show untracked files
    // TODO: 'git rev-parse --show-toplevel'

    let mut files = Vec::new();
    let ignored_files = match fs::read_to_string(".gitignore") {
        Ok(files) => files,
        Err(_) => String::new(),
    };

    let mut queue = VecDeque::new();
    queue.push_back(PathBuf::from("."));
    while !queue.is_empty() {
        let dir = &queue.pop_front().unwrap();
        let dir_name = dir.file_name();
        if dir_name.is_some() {
            let dir_name = dir_name
                .unwrap()
                .to_str()
                .expect("invalid utf-8 in dir name");
            if ignored_files.contains(&dir_name) || dir_name.contains(".git") {
                continue;
            }
        }

        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                queue.push_back(path);
            } else {
                let mut path = path.to_str().expect("invalid utf-8 in file path");
                if path.starts_with("./") {
                    path = &path[2..];
                }

                if !ignored_files.contains(&path) {
                    files.push(String::from(path));
                }
            }
        }
    }

    let index = get_entries();
    for file in &files {
        match index.iter().find(|e| file == &e.path) {
            Some(e) => {
                let file_content =
                    fs::read_to_string(Path::new(&file)).expect("cannot read file content");
                let hash = hash_object::hash_object(&file_content, &"blob", false)?;
                if e.hash != hash {
                    println!("modified: {}", file);
                }
            }
            None => println!("new: {}", file),
        };
    }

    for entry in &index {
        if files.iter().find(|&x| x == &entry.path).is_none() {
            println!("deleted: {}", entry.path);
        }
    }

    Ok(())
}
