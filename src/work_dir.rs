use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str;

use builtin::commit;
use builtin::read_tree;
use index;
use object;
use object::Object;
use refs;

#[derive(Debug)]
pub enum Error {
    CommitError(commit::Error),
    IndexError(index::Error),
    IoError(io::Error),
    ObjectError(object::Error),
    ReadTreeError(read_tree::Error),
}

impl From<commit::Error> for Error {
    fn from(e: commit::Error) -> Error {
        Error::CommitError(e)
    }
}
impl From<index::Error> for Error {
    fn from(e: index::Error) -> Error {
        Error::IndexError(e)
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}
impl From<object::Error> for Error {
    fn from(e: object::Error) -> Error {
        Error::ObjectError(e)
    }
}
impl From<read_tree::Error> for Error {
    fn from(e: read_tree::Error) -> Error {
        Error::ReadTreeError(e)
    }
}

#[derive(Debug)]
pub struct Change {
    pub state: State,
    pub path: String,
    pub hash: String,
}

#[derive(Debug, PartialEq)]
pub enum State {
    Modified,
    New,
    Deleted,
    Same,
}

pub fn diff_from_commit(oldest: &str, latest: &str) -> Result<Vec<Change>, Error> {
    let tree_hash = match oldest.is_empty() {
        true => String::new(),
        false => commit::get_tree_hash(&oldest)?,
    };
    let oldest_tree = match tree_hash.is_empty() {
        true => Vec::new(),
        false => read_tree::read_tree(&tree_hash)?,
    };
    let tree_hash = commit::get_tree_hash(&latest)?;
    let latest_tree = read_tree::read_tree(&tree_hash)?;

    let mut changes = Vec::new();
    for entry in &latest_tree {
        match oldest_tree.iter().find(|e| entry.path == e.path) {
            Some(e) => {
                let oldest_obj = Object::new(&e.hash)?;
                let latest_obj = Object::new(&entry.hash)?;
                let state = match oldest_obj.data != latest_obj.data {
                    true => State::Modified,
                    false => State::Same,
                };
                changes.push(Change {
                    state: state,
                    path: e.path.to_string(),
                    hash: e.hash.to_string(),
                });
            }

            None => changes.push(Change {
                state: State::New,
                path: entry.path.to_string(),
                hash: entry.hash.to_string(),
            }),
        }
    }

    for entry in &oldest_tree {
        let still_here = latest_tree.iter().any(|e| entry.path == e.path);
        if !still_here {
            changes.push(Change {
                state: State::Deleted,
                path: entry.path.to_string(),
                hash: entry.hash.to_string(),
            });
        }
    }

    Ok(changes)
}

pub fn update_from_commit(commit: &str) -> Result<(), Error> {
    let cur_commit = refs::get_ref_hash("HEAD")?;
    let changes = diff_from_commit(&cur_commit, &commit)?;

    let mut new_index = Vec::new();
    for change in changes {
        update_single_change(&change)?;
        if change.state != State::Deleted {
            let entry = index::Entry::new(&change.path)?;
            new_index.push(entry);
        }
    }

    index::write_entries(new_index)?;
    Ok(())
}

pub fn update_from_merge(commit1: &str, commit2: &str) -> Result<(), Error> {
    let common_ancestor = commit::lowest_common_ancestor(&commit1, &commit2)?;
    let changes1 = diff_from_commit(&common_ancestor, &commit1)?;
    let changes2 = diff_from_commit(&common_ancestor, &commit2)?;

    let mut new_index = Vec::new();
    for change in &changes1 {
        match changes2.iter().find(|c| c.path == change.path) {
            Some(c) => {
                let obj1 = Object::new(&change.hash)?;
                let obj2 = Object::new(&c.hash)?;
                if obj1.data != obj2.data {
                    // Merge conflict (no merge at all or intelligent conflict
                    // marker, just mark everything as conflict)
                    let content1 = str::from_utf8(&obj1.data).unwrap();
                    let content2 = str::from_utf8(&obj2.data).unwrap();
                    let conflict = format!(
                        "<<<<<< {}\n{}\n======\n{}\n>>>>>> {}",
                        commit1, content1, content2, commit2
                    );
                    fs::write(&change.path, conflict)?;
                }

                let entry = index::Entry::new(&change.path)?;
                new_index.push(entry);
            }
            None => {
                update_single_change(&change)?;
                if change.state != State::Deleted {
                    let entry = index::Entry::new(&change.path)?;
                    new_index.push(entry);
                }
            }
        }
    }

    for change in &changes2 {
        let not_seen = changes1.iter().all(|c| c.path != change.path);
        if not_seen {
            update_single_change(&change)?;
            if change.state != State::Deleted {
                let entry = index::Entry::new(&change.path)?;
                new_index.push(entry);
            }
        }
    }

    index::write_entries(new_index)?;
    Ok(())
}

fn update_single_change(change: &Change) -> Result<(), Error> {
    match change.state {
        State::New | State::Modified | State::Same => {
            let blob = Object::new(&change.hash)?;
            fs::write(&change.path, blob.data)?;
        }
        State::Deleted => fs::remove_file(&change.path)?,
    }

    Ok(())
}

pub fn get_all_files_path() -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(PathBuf::from("."));

    while let Some(dir) = queue.pop_front() {
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if !path.starts_with("./.git") {
                    queue.push_back(path);
                }
            } else {
                let mut path = path.to_str().unwrap();
                if path.starts_with("./") {
                    path = &path[2..];
                }

                files.push(path.to_string());
            }
        }
    }

    Ok(files)
}
