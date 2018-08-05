use std::fs;
use std::io;
use std::path::Path;

use cli;
use refs;

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

fn branch(name: &str, flag: &str) -> io::Result<()> {
    let cur_branch = refs::read_ref("HEAD")?;
    let cur_hash = refs::get_ref_hash("HEAD")?;

    if flag == "--list" || flag == "-l" || name.is_empty() {
        let refs_dir = Path::new(".git").join("refs").join("heads");
        for entry in fs::read_dir(refs_dir)? {
            let path = entry?.path();
            if path.is_file() {
                let file_name = match path.file_name() {
                    Some(p) => p.to_str().unwrap(),
                    None => continue,
                };
                if file_name == cur_branch {
                    println!("* {}", file_name);
                } else {
                    println!("  {}", file_name);
                }
            }
        }
    } else if !name.is_empty() {
        refs::write_to_ref(name, &cur_hash)?;
    }

    Ok(())
}
