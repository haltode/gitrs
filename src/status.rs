use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use hash_object;
use index;

pub fn status() -> io::Result<()> {
    // TODO: show untracked files
    // TODO: 'git rev-parse --show-toplevel'

    let index = index::get_entries().expect("cannot read index entries");
    let files = get_all_files_path().expect("cannot get stored files path");
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

fn get_all_files_path() -> io::Result<Vec<String>> {
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

    Ok(files)
}
