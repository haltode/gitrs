use std::fs;
use std::io;
use std::path::Path;

use cli;
use refs;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RefError(refs::Error),
}

pub fn cmd_branch(args: &[String], flags: &[String]) {
    let accepted_flags = ["--list", "-l"];
    if cli::has_known_flags(flags, &accepted_flags) {
        let default_val = String::new();
        let name = args.get(0).unwrap_or(&default_val);
        let flag = flags.get(0).unwrap_or(&default_val);
        if let Err(why) = branch(name, flag) {
            println!("Could not use branch: {:?}", why);
        }
    }
}

fn branch(name: &str, flag: &str) -> Result<(), Error> {
    let cur_branch = refs::head_ref().map_err(Error::RefError)?;
    let cur_hash = refs::get_ref(&cur_branch).map_err(Error::RefError)?;

    if flag == "--list" || flag == "-l" || name.is_empty() {
        let refs_dir = Path::new(".git").join("refs").join("heads");
        for entry in fs::read_dir(refs_dir).map_err(Error::IoError)? {
            let path = entry.map_err(Error::IoError)?.path();
            if path.is_dir() {
                continue;
            }

            let file_name = match path.file_name() {
                Some(p) => match p.to_str() {
                    Some(p) => p,
                    None => continue,
                },
                None => continue,
            };

            if file_name == cur_branch {
                println!("* {}", file_name);
            } else {
                println!("  {}", file_name);
            }
        }
    } else if !name.is_empty() {
        refs::write_ref(name, &cur_hash).map_err(Error::RefError)?;
    }

    Ok(())
}
