use std::fs;
use std::io;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use config;
use environment;
use hash_object;
use write_tree;

#[derive(Debug)]
pub enum Error {
    ConfigError(config::Error),
    ConfigMissing,
    HashError(hash_object::Error),
    IoError(io::Error),
    TimeError(time::SystemTimeError),
    TreeError(write_tree::Error),
    WorkingDirError(environment::Error),
}

pub fn commit(message: &str) -> Result<String, Error> {
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
    let parent_commit = git_dir.join("refs").join("heads").join("master");
    if parent_commit.exists() {
        let parent = fs::read_to_string(&parent_commit).map_err(Error::IoError)?;
        header.push_str(&format!("\nparent {}", parent));
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
         {}",
        header, author, time, author, time, message
    );

    let write = true;
    let hash = hash_object::hash_object(commit_content.as_bytes(), "commit", write)
        .map_err(Error::HashError)?;

    fs::write(&parent_commit, format!("{}\n", hash)).map_err(Error::IoError)?;
    Ok(hash)
}
