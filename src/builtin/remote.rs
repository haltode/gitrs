use std::io;

use builtin::config;

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

fn add_remote(name: &str, url: &str) -> io::Result<()> {
    let section = format!("remote.{}.url", name);
    config::config("add", &section, url)?;
    Ok(())
}
