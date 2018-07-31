use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;

use builtin::hash_object;
use index;

#[derive(Debug)]
pub enum Error {
    HashObjError(io::Error),
    IndexError(index::Error),
    IoError(io::Error),
}

enum State {
    Modified,
    New,
    Deleted,
}

pub fn cmd_status() {
    match status() {
        Ok(changes) => {
            for (state, path) in changes {
                let s = match state {
                    State::Modified => "modified",
                    State::New => "new",
                    State::Deleted => "deleted",
                };
                println!("{}: {}", s, path);
            }
        }
        Err(why) => println!("Could not retrieve status: {:?}", why),
    };
}

fn status() -> Result<Vec<(State, String)>, Error> {
    let mut status = Vec::new();
    let index = index::read_entries().map_err(Error::IndexError)?;
    let files = get_all_files_path()?;
    for file in &files {
        match index.iter().find(|e| file == &e.path) {
            Some(e) => {
                let file_content = fs::read(&file).map_err(Error::IoError)?;
                let hash = hash_object::hash_object(&file_content, "blob", false)
                    .map_err(Error::HashObjError)?;
                if e.hash != hash {
                    status.push((State::Modified, file.to_string()));
                }
            }
            None => status.push((State::New, file.to_string())),
        };
    }

    for entry in &index {
        if files.iter().all(|x| x != &entry.path) {
            status.push((State::Deleted, entry.path.to_string()));
        }
    }

    Ok(status)
}

fn get_all_files_path() -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(PathBuf::from("."));
    while let Some(dir) = queue.pop_front() {
        if let Some(dir_name) = dir.file_name() {
            if let Some(dir_name) = dir_name.to_str() {
                if dir_name.contains(".git") {
                    continue;
                }
            }
        }

        for entry in fs::read_dir(dir).map_err(Error::IoError)? {
            let path = entry.map_err(Error::IoError)?.path();
            if path.is_dir() {
                queue.push_back(path);
            } else {
                let mut path = match path.to_str() {
                    Some(p) => p,
                    None => continue,
                };
                if path.starts_with("./") {
                    path = &path[2..];
                }

                files.push(path.to_string());
            }
        }
    }

    Ok(files)
}

pub fn is_clean_working_dir() -> bool {
    match status() {
        Ok(changes) => changes.is_empty(),
        Err(_) => false,
    }
}
