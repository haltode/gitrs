use std::io;

use builtin::cat_file;
use builtin::commit;
use refs;

#[derive(Debug)]
pub enum Error {
    CatFileError(cat_file::Error),
    CommitError(commit::Error),
    RefError(io::Error),
}

pub fn cmd_log() {
    if let Err(why) = log() {
        println!("Cannot go through log: {:?}", why);
    }
}

fn log() -> Result<(), Error> {
    let mut commit_hash = refs::get_ref_hash("HEAD").map_err(Error::RefError)?;
    loop {
        println!("commit {}", commit_hash);
        cat_file::cat_file(&commit_hash, "--print").map_err(Error::CatFileError)?;

        let parents = commit::get_parents_hashes(&commit_hash).map_err(Error::CommitError)?;
        // Simple linear log, ignore multiple parents
        commit_hash = match parents.get(0) {
            Some(h) => h.to_string(),
            None => break,
        };
    }
    Ok(())
}
