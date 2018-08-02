use std::fs;
use std::io;
use std::path::Path;
use std::str;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use builtin::config;
use builtin::hash_object;
use builtin::write_tree;
use cli;
use object;
use object::Object;
use refs;

#[derive(Debug)]
pub enum Error {
    ConfigError(io::Error),
    ConfigMissing,
    HashObjError(io::Error),
    InvalidTree,
    IoError(io::Error),
    NoCommonAncestor,
    NothingToCommit,
    ObjectError(object::Error),
    RefError(io::Error),
    TimeError(time::SystemTimeError),
    TreeError(write_tree::Error),
    Utf8Error(str::Utf8Error),
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

pub fn commit(message: &str) -> Result<String, Error> {
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
    let cur_branch = refs::read_ref("HEAD").map_err(Error::RefError)?;
    if refs::exists_ref(&cur_branch) || refs::is_detached_head() {
        let cur_commit = match refs::get_ref_hash(&cur_branch) {
            Ok(r) => r,
            Err(_) => cur_branch.to_string(),
        };
        header.push_str(&format!("\nparent {}", cur_commit));

        let cur_hash = get_tree_hash(&cur_commit)?;
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
        true => Path::new(".git").join("HEAD"),
        false => Path::new(".git")
            .join("refs")
            .join("heads")
            .join(&cur_branch),
    };
    fs::write(out_dir, format!("{}\n", hash)).map_err(Error::IoError)?;

    println!("[{} {}] {}", cur_branch, &hash[..7], message);
    Ok(hash)
}

pub fn get_tree_hash(commit: &str) -> Result<String, Error> {
    let object = Object::new(&commit).map_err(Error::ObjectError)?;
    let data = str::from_utf8(&object.data).map_err(Error::Utf8Error)?;
    if !data.starts_with("tree ") || data.len() < 45 {
        return Err(Error::InvalidTree);
    }
    let tree = data[5..45].to_string();
    Ok(tree)
}

pub fn get_parent_hash(commit: &str) -> Result<String, Error> {
    let object = Object::new(&commit).map_err(Error::ObjectError)?;
    let data = str::from_utf8(&object.data).map_err(Error::Utf8Error)?;
    let data = match data.get(46..) {
        Some(d) => d,
        None => return Err(Error::InvalidTree),
    };
    let parent = match data.starts_with("parent ") {
        true => {
            if data.len() < 47 {
                return Err(Error::InvalidTree);
            } else {
                data[7..47].to_string()
            }
        }
        false => String::new(),
    };
    Ok(parent)
}

fn get_ancestors(hash: &str) -> Result<Vec<String>, Error> {
    let mut ancestors = Vec::new();
    let mut cur_commit = hash.to_string();
    while !cur_commit.is_empty() {
        cur_commit = get_parent_hash(&cur_commit)?;
        ancestors.push(cur_commit.clone());
    }

    Ok(ancestors)
}

pub fn is_ancestor(commit1: &str, commit2: &str) -> bool {
    let commit1_ancestors = match get_ancestors(&commit1) {
        Ok(a) => a,
        Err(_) => return false,
    };

    commit1_ancestors.contains(&commit2.to_string())
}

pub fn lowest_common_ancestor(commit1: &str, commit2: &str) -> Result<String, Error> {
    let commit1_ancestors = get_ancestors(&commit1)?;
    let commit2_ancestors = get_ancestors(&commit2)?;

    for ancestor in commit1_ancestors {
        if commit2_ancestors.contains(&ancestor) {
            return Ok(ancestor);
        }
    }

    Err(Error::NoCommonAncestor)
}
