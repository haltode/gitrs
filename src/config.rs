use std::fs;
use std::io;
use std::path::Path;

pub struct Config {
    pub name: String,
    pub email: String,
}

pub fn parse_config() -> io::Result<Config> {
    let config_file = Path::new(".git").join("config");
    let mut name = String::new();
    let mut email = String::new();
    if config_file.exists() {
        let data = fs::read_to_string(config_file)?;
        for line in data.lines().map(|l| l.trim()) {
            let ele: Vec<&str> = line.split('=').collect();
            if ele.len() != 2 {
                continue;
            }

            let section = ele[0].trim();
            let value = ele[1].trim_left().to_string();
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

pub fn config(option: &str, section: &str, value: &str) -> io::Result<()> {
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
        opt => println!("config: unknown option '{}'", opt),
    }

    if modif {
        let config_fmt = format!("[user]\n\tname = {}\n\temail = {}\n", user.name, user.email);
        let config_file = Path::new(".git").join("config");
        fs::write(config_file, config_fmt)?;
    }

    Ok(())
}
