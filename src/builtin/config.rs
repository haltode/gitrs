use std::fs;
use std::io;

use cli;
use environment;

pub struct Config {
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    WorkingDirError(environment::Error),
}

pub fn cmd_config(args: &[String], flags: &[String]) {
    let accepted_flags = ["--add", "--get", "--unset", "--list"];
    if cli::has_known_flags(flags, &accepted_flags) {
        if flags.is_empty() {
            println!("config: command takes option such as '--add', '--list', etc.");
        } else {
            let default_val = String::new();
            let section = args.get(0).unwrap_or(&default_val);
            let value = args.get(1).unwrap_or(&default_val);
            let option = &flags[0][2..];

            if let Err(why) = config(option, section, value) {
                println!("Could not use config file: {:?}", why);
            }
        }
    }
}

pub fn parse_config() -> Result<Config, Error> {
    let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
    let config_file = git_dir.join("config");
    let mut name = String::new();
    let mut email = String::new();
    if config_file.exists() {
        let data = fs::read_to_string(config_file).map_err(Error::IoError)?;
        for line in data.lines().map(|l| l.trim()) {
            let elem: Vec<&str> = line.split('=').collect();
            if elem.len() != 2 {
                continue;
            }

            let section = elem[0].trim();
            let value = elem[1].trim().to_string();
            if section == "name" {
                name = value;
            } else if section == "email" {
                email = value;
            }
        }
    }

    Ok(Config {
        name: name,
        email: email,
    })
}

fn config(option: &str, section: &str, value: &str) -> Result<(), Error> {
    let mut user = parse_config()?;
    let mut modif = false;
    match option {
        "add" => {
            modif = true;
            match section {
                "user.name" => user.name = value.to_string(),
                "user.email" => user.email = value.to_string(),
                sct => println!("config: unknown section '{}'", sct),
            }
        }

        "get" => match section {
            "user.name" => println!("{}", user.name),
            "user.email" => println!("{}", user.email),
            sct => println!("config: unknown section '{}'", sct),
        },

        "unset" => {
            modif = true;
            match section {
                "user.name" => user.name = String::new(),
                "user.email" => user.email = String::new(),
                sct => println!("config: unknown section '{}'", sct),
            }
        }

        "list" => {
            println!("user.name = {}", user.name);
            println!("user.email = {}", user.email);
        }

        _ => unreachable!(),
    }

    if modif {
        let config_fmt = format!("[user]\n\tname = {}\n\temail = {}\n", user.name, user.email);
        let git_dir = environment::get_working_dir().map_err(Error::WorkingDirError)?;
        let config_file = git_dir.join("config");
        fs::write(config_file, config_fmt).map_err(Error::IoError)?;
    }

    Ok(())
}
