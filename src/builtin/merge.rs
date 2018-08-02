use std::fs;
use std::io;
use std::path::Path;

use builtin::commit;
use builtin::status;
use object;
use refs;
use working_dir;

#[derive(Debug)]
pub enum Error {
    AlreadyUpToDate,
    CommitError(commit::Error),
    IoError(io::Error),
    ObjectError(object::Error),
    RefError(io::Error),
    ReferenceNotACommit,
    WorkingDirError(working_dir::Error),
    WorkingDirNotClean,
}

pub fn cmd_merge(args: &[String]) {
    if args.is_empty() {
        println!("merge: command takes a 'ref' argument.");
    } else {
        let ref_name = &args[0];
        if let Err(why) = merge(ref_name) {
            println!("Could not merge: {:?}", why);
        }
    }
}

fn merge(ref_name: &str) -> Result<(), Error> {
    if !status::is_clean_working_dir() {
        return Err(Error::WorkingDirNotClean);
    }

    let cur_commit = refs::get_ref_hash("HEAD").map_err(Error::RefError)?;
    let dst_commit = refs::get_ref_hash(&ref_name).map_err(Error::RefError)?;
    if cur_commit == dst_commit {
        return Err(Error::AlreadyUpToDate);
    }

    let object = object::get_object(&dst_commit).map_err(Error::ObjectError)?;
    if object.obj_type != "commit" {
        return Err(Error::ReferenceNotACommit);
    }

    let cur_branch = refs::read_ref("HEAD").map_err(Error::RefError)?;
    let can_fast_forward = commit::is_ancestor(&dst_commit, &cur_commit);
    if can_fast_forward {
        working_dir::update_from_commit(&dst_commit).map_err(Error::WorkingDirError)?;

        let branch_path = Path::new(".git")
            .join("refs")
            .join("heads")
            .join(&cur_branch);
        fs::write(branch_path, format!("{}\n", dst_commit)).map_err(Error::IoError)?;
        println!("Fast-forward");
    } else {
        working_dir::update_from_merge(&cur_commit, &dst_commit).map_err(Error::WorkingDirError)?;
        let merge_msg = format!("Merge {} into {}", ref_name, cur_branch);
        println!("{}", merge_msg);

        let mut has_conflicts = false;
        for file in working_dir::get_all_files_path().map_err(Error::IoError)? {
            let data = fs::read_to_string(&file).map_err(Error::IoError)?;
            if data.contains("<<<<<<") {
                println!("CONFLICT {}", file);
                has_conflicts = true;
            }
        }

        if !has_conflicts {
            commit::commit(&merge_msg).map_err(Error::CommitError)?;
        } else {
            println!("Conflicts detected, fix them and commit to finish merge:");
        }
    }

    Ok(())
}
