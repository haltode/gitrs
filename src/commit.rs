use std::fs;
use std::io;
use std::path::Path;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use config;
use hash_object;
use write_tree;

#[derive(Debug)]
pub enum Error {
    ConfigError(io::Error),
    HashError(io::Error),
    IoError(io::Error),
    TimeError(time::SystemTimeError),
    TreeError(write_tree::Error),
}

pub fn commit(message: &str) -> Result<String, Error> {
    let tree = write_tree::write_tree().map_err(Error::TreeError)?;
    let mut header = format!("tree {}", tree);
    let parent_commit = Path::new(".git").join("refs").join("heads").join("master");
    if parent_commit.exists() {
        let parent = fs::read_to_string(&parent_commit).map_err(Error::IoError)?;
        header.push_str(&format!("\nparent {}", parent));
    }

    let user = config::parse_config().map_err(Error::ConfigError)?;
    let author = format!("{} <{}>", user.name, user.email);

    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH).map_err(Error::TimeError)?;
    // I decide where you live, alright?!
    let timezone = "+0200";
    let time = format!("{} {}", timestamp.as_secs(), timezone);

    let commit_content = format!(
        "{}\n\
         author {} {}\n\
         committer {} {}\n\n\
         {}",
        header, author, time, author, time, message
    );

    let write = true;
    let hash = hash_object::hash_object(commit_content.as_bytes(), "commit", write)
        .map_err(Error::HashError)?;

    fs::write(&parent_commit, format!("{}\n", hash)).map_err(Error::IoError)?;
    Ok(hash)
}
