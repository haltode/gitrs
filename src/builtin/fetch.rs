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
    RemoteNotAGitRepo,
    RemoteNotFound,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

pub fn cmd_fetch(args: &[String]) {
    if args.len() < 2 {
        println!("fetch: takes 'remote' and 'branch' arguments");
    } else {
        let remote = &args[0];
        let branch = &args[1];
        if let Err(why) = fetch(&remote, &branch) {
            println!("Could not fetch: {:?}", why);
        }
    }
}

pub fn fetch(remote: &str, branch: &str) -> Result<(), Error> {
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
    }
    let remote_hash = refs::get_ref_hash(&branch)?;
    if local_hash == remote_hash {
        return Err(Error::AlreadyUpToDate);
    }

    let missing = remote::find_remote_missing_objects(&remote_hash, &local_hash);
    for obj_hash in &missing {
        let obj = object::Object::new(&obj_hash).map_err(Error::ObjectError)?;

        env::set_current_dir(&local_dir)?;
        let write = true;
        hash_object::hash_object(&obj.data, &obj.obj_type, write)?;
        env::set_current_dir(&remote_dir)?;
    }
    env::set_current_dir(&local_dir)?;

    let rem_dir = Path::new(".git").join("refs").join("remotes").join(&remote);
    fs::create_dir_all(&rem_dir)?;
    fs::write(rem_dir.join(&branch), format!("{}\n", remote_hash))?;

    let fetch_head = Path::new(".git").join("FETCH_HEAD");
    fs::write(
        fetch_head,
        format!("{} branch '{}' of {}\n", remote_hash, branch, url),
    )?;

    println!("Count: {} objects", missing.len());
    println!("From: {}", url);
    Ok(())
}
