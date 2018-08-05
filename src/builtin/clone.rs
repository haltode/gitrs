use std::env;
use std::fs;
use std::io;
use std::path::Path;

use builtin::init;
use builtin::pull;
use builtin::remote;
use refs;

#[derive(Debug)]
pub enum Error {
    DirectoryAlreadyExists,
    IoError(io::Error),
    NotAGitRepository,
    PullError(pull::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

pub fn cmd_clone(args: &[String]) {
    if args.len() < 2 {
        println!("clone: takes 'repository' and 'directory' arguments");
    } else {
        let repository = &args[0];
        let directory = &args[1];
        if let Err(why) = clone(&repository, &directory) {
            println!("Could not clone: {:?}", why);
        }
    }
}

fn clone(repository: &str, directory: &str) -> Result<(), Error> {
    let repo_path = Path::new(&repository);
    let dir_path = Path::new(&directory);

    if !repo_path.join(".git").exists() {
        return Err(Error::NotAGitRepository);
    }
    if dir_path.exists() {
        return Err(Error::DirectoryAlreadyExists);
    }

    init::init(&directory)?;

    let absolute_repo_path = fs::canonicalize(&repo_path)?;
    let absolute_dir_path = fs::canonicalize(&dir_path)?;

    env::set_current_dir(&absolute_repo_path)?;
    let has_commits = refs::get_ref_hash("HEAD").is_ok();

    env::set_current_dir(&absolute_dir_path)?;
    remote::add_remote("origin", absolute_repo_path.to_str().unwrap())?;
    if has_commits {
        pull::pull("origin", "master").map_err(Error::PullError)?;
    }

    println!("Cloning into {}", directory);
    Ok(())
}
