use std::fs;
use std::io;

use builtin::hash_object;
use index;
use work_dir;

#[derive(Debug)]
pub enum Error {
    IndexError(index::Error),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
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
    let files = work_dir::get_all_files_path()?;
    for file in &files {
        match index.iter().find(|e| file == &e.path) {
            Some(e) => {
                let file_content = fs::read(&file)?;
                let hash = hash_object::hash_object(&file_content, "blob", false)?;
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

pub fn is_clean_work_dir() -> bool {
    match status() {
        Ok(changes) => changes.is_empty(),
        Err(_) => false,
    }
}
