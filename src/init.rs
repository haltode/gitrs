use std::fs;
use std::io;
use std::path::Path;

pub fn init(dir_name: &str) -> io::Result<()> {
    let dir_path = Path::new(dir_name);
    let git_path = Path::new(&dir_path).join(".git");

    if !dir_name.is_empty() {
        fs::create_dir(&dir_path)?;
    }
    fs::create_dir(&git_path)?;
    for dir in ["objects", "refs", "refs/heads"].iter() {
        fs::create_dir(Path::new(&git_path).join(dir))?;
    }
    fs::write(Path::new(&git_path).join("HEAD"), "ref: refs/heads/master")?;

    println!("Initialized empty Git repository in {}", git_path.display());
    Ok(())
}
