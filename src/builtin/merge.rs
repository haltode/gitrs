use std::fs;
use std::io;
use std::path::Path;

use builtin::checkout;
use builtin::commit;
use builtin::status;
use object;
use refs;

#[derive(Debug)]
pub enum Error {
    AlreadyUpToDate,
    IoError(io::Error),
    ObjectError(object::Error),
    RefError(io::Error),
    ReferenceNotACommit,
    TreeCommitError(commit::Error),
    UpdateDirError(checkout::Error),
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

    let can_fast_forward = commit::is_ancestor(&dst_commit, &cur_commit);
    if can_fast_forward {
        let tree = commit::get_tree(&dst_commit).map_err(Error::TreeCommitError)?;
        checkout::update_working_dir(&dst_commit, &tree).map_err(Error::UpdateDirError)?;

        let cur_branch = refs::read_ref("HEAD").map_err(Error::RefError)?;
        let branch_path = Path::new(".git")
            .join("refs")
            .join("heads")
            .join(&cur_branch);
        fs::write(branch_path, format!("{}\n", dst_commit)).map_err(Error::IoError)?;
    } else {

    }

    Ok(())
}
