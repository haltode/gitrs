use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str;

use builtin::commit;
use builtin::read_tree;
use index;
use object;
use refs;

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

#[derive(Debug)]
pub enum Error {
    CommitError(commit::Error),
    IndexError(index::Error),
    IoError(io::Error),
    ObjectError(object::Error),
    ReadTreeError(read_tree::Error),
    RefError(io::Error),
    Utf8Error(str::Utf8Error),
}

pub fn diff_from_commit(oldest: &str, latest: &str) -> Result<Vec<Change>, Error> {
    let tree_hash = commit::get_tree(&oldest).map_err(Error::CommitError)?;
    let oldest_tree = read_tree::read_tree(&tree_hash).map_err(Error::ReadTreeError)?;
    let tree_hash = commit::get_tree(&latest).map_err(Error::CommitError)?;
    let latest_tree = read_tree::read_tree(&tree_hash).map_err(Error::ReadTreeError)?;

    let mut changes = Vec::new();
    for entry in &latest_tree {
        match oldest_tree.iter().find(|e| entry.path == e.path) {
            Some(e) => {
                let oldest_obj = object::get_object(&e.hash).map_err(Error::ObjectError)?;
                let latest_obj = object::get_object(&entry.hash).map_err(Error::ObjectError)?;
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
    let cur_commit = refs::get_ref_hash("HEAD").map_err(Error::RefError)?;
    let changes = diff_from_commit(&cur_commit, &commit)?;

    let mut new_index = Vec::new();
    for change in changes {
        update_single_change(&change)?;
        if change.state != State::Deleted {
            let entry = index::Entry::new(&change.path).map_err(Error::IndexError)?;
            new_index.push(entry);
        }
    }

    index::write_entries(new_index).map_err(Error::IndexError)?;
    Ok(())
}

pub fn update_from_merge(commit1: &str, commit2: &str) -> Result<(), Error> {
    let common_ancestor =
        commit::lowest_common_ancestor(&commit1, &commit2).map_err(Error::CommitError)?;
    let changes1 = diff_from_commit(&common_ancestor, &commit1)?;
    let changes2 = diff_from_commit(&common_ancestor, &commit2)?;

    let mut new_index = Vec::new();
    for change in &changes1 {
        match changes2.iter().find(|c| c.path == change.path) {
            Some(c) => {
                let obj1 = object::get_object(&change.hash).map_err(Error::ObjectError)?;
                let obj2 = object::get_object(&c.hash).map_err(Error::ObjectError)?;
                if obj1.data != obj2.data {
                    // Merge conflict (no merge at all or intelligent conflict
                    // marker, just mark everything as conflict)
                    let content1 = str::from_utf8(&obj1.data).map_err(Error::Utf8Error)?;
                    let content2 = str::from_utf8(&obj2.data).map_err(Error::Utf8Error)?;
                    let conflict = format!(
                        "<<<<<< {}\n{}\n======\n{}\n>>>>>> {}",
                        commit1, content1, content2, commit2
                    );
                    fs::write(&change.path, conflict).map_err(Error::IoError)?;
                }

                let entry = index::Entry::new(&change.path).map_err(Error::IndexError)?;
                new_index.push(entry);
            }
            None => {
                update_single_change(&change)?;
                if change.state != State::Deleted {
                    let entry = index::Entry::new(&change.path).map_err(Error::IndexError)?;
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
                let entry = index::Entry::new(&change.path).map_err(Error::IndexError)?;
                new_index.push(entry);
            }
        }
    }

    index::write_entries(new_index).map_err(Error::IndexError)?;
    Ok(())
}

fn update_single_change(change: &Change) -> Result<(), Error> {
    match change.state {
        State::New | State::Modified | State::Same => {
            let blob = object::get_object(&change.hash).map_err(Error::ObjectError)?;
            fs::write(&change.path, blob.data).map_err(Error::IoError)?;
        }
        State::Deleted => fs::remove_file(&change.path).map_err(Error::IoError)?,
    }

    Ok(())
}

pub fn get_all_files_path() -> io::Result<Vec<String>> {
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

        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
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