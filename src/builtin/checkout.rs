use std::fs;
use std::io;
use std::str;

use builtin::read_tree;
use environment;
use index;
use object;
use refs;

#[derive(Debug)]
pub enum Error {
    AlreadyOnIt,
    IndexError(index::Error),
    IoError(io::Error),
    ObjectError(object::Error),
    RefError(refs::Error),
    ReferenceNotACommit,
    TreeError(read_tree::Error),
    Utf8Error(str::Utf8Error),
    WorkingDirError(environment::Error),
}

pub fn cmd_checkout(args: &[String]) {
    if args.is_empty() {
        println!("checkout: command takes a 'ref' argument.");
    } else {
        let ref_name = &args[0];
        if let Err(why) = checkout(ref_name) {
            println!("Could not checkout: {:?}", why);
        }
    }
}

fn checkout(ref_name: &str) -> Result<(), Error> {
    let cur_commit = match refs::get_ref(&ref_name) {
        Ok(r) => r,
        Err(_) => ref_name.to_string(),
    };
    let is_detached_head = cur_commit == ref_name;

    let object = object::get_object(&cur_commit).map_err(Error::ObjectError)?;
    if object.obj_type != "commit" {
        return Err(Error::ReferenceNotACommit);
    }

    let head = refs::head_ref().map_err(Error::RefError)?;
    if ref_name == head {
        return Err(Error::AlreadyOnIt);
    }

    let tree = object::get_tree_from_commit(&cur_commit).map_err(Error::ObjectError)?;
    update_working_dir(ref_name, &tree)?;

    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    let head_dir = git_dir.join("HEAD");
    if is_detached_head {
        fs::write(head_dir, format!("{}\n", ref_name)).map_err(Error::IoError)?;
    } else {
        let full_ref = refs::format_ref_name(&ref_name);
        fs::write(head_dir, format!("ref: {}\n", full_ref)).map_err(Error::IoError)?;
    }

    if is_detached_head {
        println!("Note: checking out {}", ref_name);
        println!("You are in detached HEAD state.");
    } else {
        println!("Switched to branch {}", ref_name);
    }
    Ok(())
}

fn update_working_dir(ref_name: &str, tree_hash: &str) -> Result<(), Error> {
    let mut new_index = Vec::new();
    let tree = read_tree::read_tree(tree_hash).map_err(Error::TreeError)?;
    let index = index::read_entries().map_err(Error::IndexError)?;

    // Check for addition and modification
    for entry in &tree {
        let blob = object::get_object(&entry.hash).map_err(Error::ObjectError)?;
        match index.iter().find(|e| entry.path == e.path) {
            // Modif (no merge at all or intelligent conflict marker, just mark everything as conflict)
            Some(e) => {
                let blob_data = str::from_utf8(&blob.data).map_err(Error::Utf8Error)?;
                let head_data = fs::read_to_string(&entry.path).map_err(Error::IoError)?;
                if head_data != blob_data {
                    let conflict = format!(
                        "<<<<<< HEAD\n{}======\n{}>>>>>> {}\n",
                        head_data, blob_data, ref_name
                    );
                    fs::write(&entry.path, conflict).map_err(Error::IoError)?;
                }

                new_index.push(e.clone());
            }

            // Add
            None => {
                fs::write(&entry.path, blob.data).map_err(Error::IoError)?;
                let new_entry = index::Entry::new(&entry.path).map_err(Error::IndexError)?;
                new_index.push(new_entry);
            }
        };
    }

    // Check for deletion
    for entry in &index {
        let in_tree = tree.iter().any(|e| e.path == entry.path);
        if !in_tree {
            fs::remove_file(&entry.path).map_err(Error::IoError)?;
        }
    }

    index::write_entries(new_index).map_err(Error::IndexError)?;
    Ok(())
}
