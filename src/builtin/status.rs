use std::collections::VecDeque;
use std::fs;
use std::io;

use builtin::hash_object;
use environment;
use index;

#[derive(Debug)]
pub enum Error {
    HashError(hash_object::Error),
    IndexError(index::Error),
    IoError(io::Error),
    WorkingDirError(environment::Error),
}

pub fn cmd_status() {
    if let Err(why) = status() {
        println!("Could not retrieve status: {:?}", why);
    }
}

fn status() -> Result<(), Error> {
    let index = index::read_entries().map_err(Error::IndexError)?;
    let files = get_all_files_path()?;
    let mut hashes = Vec::new();
    for file in &files {
        let file_content = fs::read(&file).map_err(Error::IoError)?;
        let hash =
            hash_object::hash_object(&file_content, "blob", false).map_err(Error::HashError)?;
        hashes.push(hash.to_string());

        if index.iter().any(|e| hash == e.hash) {
            println!("modified: {}", file);
        } else {
            println!("new: {}", file);
        }
    }

    for entry in &index {
        if !hashes.contains(&entry.hash) {
            println!("deleted: {}", entry.path);
        }
    }

    Ok(())
}

fn get_all_files_path() -> Result<Vec<String>, Error> {
    // Start out from top directory level
    let mut git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    git_dir.pop();

    let mut files = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(git_dir);
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
