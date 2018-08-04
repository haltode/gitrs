use std::io;

use builtin::config;
use object;

#[derive(Debug)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

pub fn cmd_remote(args: &[String]) {
    if args.is_empty() {
        if let Err(why) = list_remotes() {
            println!("Could not list remotes: {}", why);
        }
    } else {
        let cmd = &args[0];
        if cmd == "add" {
            if args.len() < 3 {
                println!("remote: 'add' command takes 'name' and 'url' arguments");
            } else {
                let name = &args[1];
                let url = &args[2];
                if let Err(why) = add_remote(name, url) {
                    println!("Could not add remote: {}", why);
                }
            }
        } else {
            println!("remote: unknown command '{}'", cmd);
        }
    }
}

fn list_remotes() -> io::Result<()> {
    let user = config::Config::new()?;
    for remote in user.remotes {
        println!("{} {}", remote.name, remote.url);
    }
    Ok(())
}

pub fn add_remote(name: &str, url: &str) -> io::Result<()> {
    let section = format!("remote.{}.url", name);
    config::config("add", &section, url)?;
    Ok(())
}

pub fn find_remote_missing_objects(local_commit: &str, remote_commit: &str) -> Vec<String> {
    let local_objects = object::find_objects_from_commit(&local_commit);
    let remote_objects = object::find_objects_from_commit(&remote_commit);

    let mut missing = Vec::new();
    for obj in local_objects {
        if !remote_objects.contains(&obj) {
            missing.push(obj);
        }
    }

    missing
}
