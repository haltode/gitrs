use std::fs;
use std::io;
use std::path::Path;
use std::str;
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
    CannotGetParentFromCommit,
    CannotGetTreeFromCommit,
    IoError(io::Error),
    NoCommonAncestor,
    NothingToCommit,
    ObjectError(object::Error),
    TreeError(write_tree::Error),
    UserConfigIncomplete,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
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
    let user = config::Config::new()?;
    if user.name.is_empty() || user.email.is_empty() {
        return Err(Error::UserConfigIncomplete);
    }
    let author = format!("{} <{}>", user.name, user.email);

    let commit_tree = write_tree::write_tree().map_err(Error::TreeError)?;
    let mut parents = Vec::new();

    let head = refs::read_ref("HEAD")?;
    let has_commits = refs::exists_ref(&head) || refs::is_detached_head();
    if has_commits {
        let cur_commit = match refs::get_ref_hash(&head) {
            Ok(r) => r,
            Err(_) => head.to_string(),
        };
        parents.push(cur_commit.to_string());

        let merge_head = Path::new(".git").join("MERGE_HEAD");
        let is_in_merge = merge_head.exists();
        if is_in_merge {
            let merge_parent = fs::read_to_string(&merge_head)?;
            parents.push(merge_parent);
            fs::remove_file(&merge_head)?;
        }

        let cur_hash = get_tree_hash(&cur_commit)?;
        if commit_tree == cur_hash && !is_in_merge {
            println!("On {}", head);
            println!("nothing to commit, working tree clean");
            return Err(Error::NothingToCommit);
        }
    }

    let mut header = format!("tree {}", commit_tree);
    for parent in parents {
        header.push_str(&format!("\nparent {}", parent));
    }

    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH).unwrap();
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
    let hash = hash_object::hash_object(commit_content.as_bytes(), "commit", write)?;

    let ref_path = match refs::is_detached_head() {
        true => Path::new(".git").join("HEAD"),
        false => Path::new(".git").join("refs").join("heads").join(&head),
    };
    fs::write(ref_path, format!("{}\n", hash))?;

    println!("[{} {}] {}", head, &hash[..7], message);
    Ok(hash)
}

pub fn get_tree_hash(commit: &str) -> Result<String, Error> {
    let object = Object::new(&commit).map_err(Error::ObjectError)?;
    let data = str::from_utf8(&object.data).unwrap();
    if !data.starts_with("tree ") || data.len() < 45 {
        return Err(Error::CannotGetTreeFromCommit);
    }
    let tree = data[5..45].to_string();
    Ok(tree)
}

pub fn get_parents_hashes(commit: &str) -> Result<Vec<String>, Error> {
    let object = Object::new(&commit).map_err(Error::ObjectError)?;
    let data = str::from_utf8(&object.data).unwrap();

    let mut parents = Vec::new();
    let prefix = "parent ";
    for line in data.lines().filter(|l| l.starts_with(prefix)) {
        if line.len() != prefix.len() + 40 {
            return Err(Error::CannotGetParentFromCommit);
        }

        let start = prefix.len();
        let end = start + 40;
        let hash = &line[start..end];
        parents.push(hash.to_string());
    }

    Ok(parents)
}

fn get_ancestors(commit: &str) -> Result<Vec<String>, Error> {
    let mut ancestors = Vec::new();
    for parent in get_parents_hashes(&commit)? {
        ancestors.push(parent.to_string());
        ancestors.extend(get_ancestors(&parent)?);
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
