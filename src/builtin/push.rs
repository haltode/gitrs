use std::env;
use std::fs;
use std::io;
use std::path::Path;

use builtin::config;
use builtin::hash_object;
use builtin::remote;
use object;
use refs;

#[derive(Debug)]
pub enum Error {
    AlreadyUpToDate,
    IoError(io::Error),
    ObjectError(object::Error),
    RemoteBranchCurrentlyCheckedOut,
    RemoteNotAGitRepo,
    RemoteNotFound,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

pub fn cmd_push(args: &[String]) {
    if args.len() < 2 {
        println!("push: takes 'remote' and 'branch' arguments");
    } else {
        let remote = &args[0];
        let branch = &args[1];
        if let Err(why) = push(&remote, &branch) {
            println!("Could not push: {:?}", why);
        }
    }
}

fn push(remote: &str, branch: &str) -> Result<(), Error> {
    let user = config::Config::new()?;
    let url = match user.remotes.iter().find(|r| r.name == remote) {
        Some(r) => r.url.to_string(),
        None => return Err(Error::RemoteNotFound),
    };

    let local_dir = env::current_dir()?;
    let local_hash = refs::get_ref_hash(&branch)?;
    let remote_dir = Path::new(&url);

    env::set_current_dir(&remote_dir)?;
    if !Path::new(".git").exists() {
        return Err(Error::RemoteNotAGitRepo);
    } else {
        let is_bare_repo = config::Config::new()?.is_empty();
        if !is_bare_repo && refs::read_ref("HEAD")? == branch {
            return Err(Error::RemoteBranchCurrentlyCheckedOut);
        }
    }

    let remote_hash = refs::get_ref_hash(&branch)?;
    if local_hash == remote_hash {
        return Err(Error::AlreadyUpToDate);
    } else {
        refs::write_to_ref(&branch, &local_hash)?;
    }

    env::set_current_dir(&local_dir)?;
    let missing = remote::find_remote_missing_objects(&local_hash, &remote_hash);
    for obj_hash in &missing {
        let obj = object::Object::new(&obj_hash).map_err(Error::ObjectError)?;

        env::set_current_dir(&remote_dir)?;
        let write = true;
        hash_object::hash_object(&obj.data, &obj.obj_type, write)?;
        env::set_current_dir(&local_dir)?;
    }

    let rem_dir = Path::new(".git").join("refs").join("remotes").join(&remote);
    fs::create_dir_all(&rem_dir)?;
    fs::write(rem_dir.join(&branch), format!("{}\n", local_hash))?;

    println!("Count: {} objects", missing.len());
    println!("To: {}", url);
    Ok(())
}
