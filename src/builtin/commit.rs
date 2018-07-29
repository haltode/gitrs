use std::fs;
use std::io;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use builtin::config;
use builtin::hash_object;
use builtin::write_tree;
use cli;
use environment;
use object;
use refs;

#[derive(Debug)]
pub enum Error {
    ConfigError(config::Error),
    ConfigMissing,
    HashObjError(hash_object::Error),
    IoError(io::Error),
    NothingToCommit,
    ObjectError(object::Error),
    RefError(refs::Error),
    TimeError(time::SystemTimeError),
    TreeError(write_tree::Error),
    WorkingDirError(environment::Error),
}

pub fn cmd_commit(args: &[String], flags: &[String]) {
    let accepted_flags = ["--message", "-m"];
    if cli::has_known_flags(flags, &accepted_flags) {
        if cli::has_flag(flags, "--message", "-m") {
            let message = &args[0];
            if let Err(why) = commit(message) {
                println!("Could not commit: {:?}", why);
            }
        } else {
            println!("commit: command needs a '--message' flag.");
        }
    }
}

fn commit(message: &str) -> Result<String, Error> {
    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;

    let user = config::parse_config().map_err(Error::ConfigError)?;
    if user.name.is_empty() || user.email.is_empty() {
        println!("Need to specify your name/email before committing:");
        println!("\tgitrs config --add user.name your_name");
        println!("\tgitrs config --add user.email your_email");
        return Err(Error::ConfigMissing);
    }
    let author = format!("{} <{}>", user.name, user.email);

    let tree = write_tree::write_tree().map_err(Error::TreeError)?;
    let mut header = format!("tree {}", tree);
    let cur_branch = refs::head_ref().map_err(Error::RefError)?;
    if refs::exists_ref(&cur_branch) || refs::is_detached_head() {
        let cur_commit = match refs::get_ref(&cur_branch) {
            Ok(r) => r,
            Err(_) => cur_branch.to_string(),
        };
        header.push_str(&format!("\nparent {}", cur_commit));

        let cur_hash = object::get_tree_from_commit(&cur_commit).map_err(Error::ObjectError)?;
        if tree == cur_hash {
            println!("On {}", cur_branch);
            println!("nothing to commit, working tree clean");
            return Err(Error::NothingToCommit);
        }
    }

    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH).map_err(Error::TimeError)?;
    // I decide where you live, alright?!
    let timezone = "+0200";
    let time = format!("{} {}", timestamp.as_secs(), timezone);

    let commit_content = format!(
        "{}\n\
         author {} {}\n\
         committer {} {}\n\n\
         {}\n",
        header, author, time, author, time, message
    );

    let write = true;
    let hash = hash_object::hash_object(commit_content.as_bytes(), "commit", write)
        .map_err(Error::HashObjError)?;

    let out_dir = match refs::is_detached_head() {
        true => git_dir.join("HEAD"),
        false => git_dir.join("refs").join("heads").join(&cur_branch),
    };
    fs::write(out_dir, format!("{}\n", hash)).map_err(Error::IoError)?;

    println!("[{} {}] {}", cur_branch, &hash[..7], message);
    Ok(hash)
}
