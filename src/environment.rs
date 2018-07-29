use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    WorkingDirNotFound,
}

pub fn get_working_dir() -> Result<PathBuf, Error> {
    let mut work_dir = PathBuf::from(".");
    loop {
        for entry in fs::read_dir(&work_dir).map_err(Error::IoError)? {
            let path = entry.map_err(Error::IoError)?.path();
            if !path.is_dir() {
                continue;
            }

            if let Some(dir_name) = path.file_name() {
                if dir_name.to_str() == Some(".git") {
                    work_dir = work_dir.join(".git");
                    return Ok(work_dir);
                }
            }
        }

        work_dir = work_dir.join("..");
        if !work_dir.exists() {
            break;
        }
    }

    Err(Error::WorkingDirNotFound)
}
