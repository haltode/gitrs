use std::fs;
use std::io;

use builtin::hash_object;
use index;
use working_dir;

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
    let files = working_dir::get_all_files_path().map_err(Error::IoError)?;
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

pub fn is_clean_working_dir() -> bool {
    match status() {
        Ok(changes) => changes.is_empty(),
        Err(_) => false,
    }
}
