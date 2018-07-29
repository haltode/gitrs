use std::fs;
use std::io;

use environment;

#[derive(Debug)]
pub enum Error {
    InvalidHEADFile,
    IoError(io::Error),
    WorkingDirError(environment::Error),
}

pub fn format_ref_name(name: &str) -> String {
    match name.starts_with("refs/heads/") {
        true => name.to_string(),
        false => format!("refs/heads/{}", name),
    }
}

pub fn get_ref(name: &str) -> Result<String, Error> {
    let ref_name = format_ref_name(name);
    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    let mut val = fs::read_to_string(git_dir.join(ref_name)).map_err(Error::IoError)?;
    // Remove '\n' character
    val.pop();
    Ok(val)
}

pub fn write_ref(name: &str, value: &str) -> Result<(), Error> {
    let ref_name = format_ref_name(name);
    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    fs::write(git_dir.join(ref_name), format!("{}\n", value)).map_err(Error::IoError)?;
    Ok(())
}

pub fn exists_ref(name: &str) -> bool {
    let ref_name = format_ref_name(name);
    let git_dir = match environment::get_working_dir() {
        Ok(d) => d,
        Err(_) => return false,
    };
    git_dir.join(ref_name).exists()
}

pub fn head_ref() -> Result<String, Error> {
    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    let mut head = fs::read_to_string(git_dir.join("HEAD")).map_err(Error::IoError)?;
    // Remove '\n' character
    head.pop();

    if head.starts_with("ref: refs/heads/") {
        let branch = match head.get(16..) {
            Some(b) => b.to_string(),
            None => return Err(Error::InvalidHEADFile),
        };
        return Ok(branch);
    } else {
        return Ok(head);
    }
}

pub fn is_detached_head() -> bool {
    let git_dir = match environment::get_working_dir() {
        Ok(d) => d,
        Err(_) => return false,
    };
    let head = match fs::read_to_string(git_dir.join("HEAD")) {
        Ok(s) => s,
        Err(_) => return false,
    };
    !head.starts_with("ref: ")
}
