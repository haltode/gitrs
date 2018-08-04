use std::fs;
use std::io;
use std::path::Path;

pub fn cmd_init(args: &[String]) {
    let default_path = String::new();
    let path = args.get(0).unwrap_or(&default_path);
    if let Err(why) = init(path) {
        println!("Could not initialize git repository: {:?}", why);
    }
}

pub fn init(dir_name: &str) -> io::Result<()> {
    if !dir_name.is_empty() {
        fs::create_dir(&dir_name)?;
    }

    let git_path = Path::new(&dir_name).join(".git");
    fs::create_dir(&git_path)?;
    for dir in ["objects", "refs", "refs/heads", "refs/remotes"].iter() {
        fs::create_dir(git_path.join(dir))?;
    }
    fs::write(git_path.join("HEAD"), "ref: refs/heads/master\n")?;

    println!("Initialized empty Git repository in {}", git_path.display());
    Ok(())
}
