use std::fs;
use std::io;
use std::path::Path;

use builtin::remote;
use cli;

#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub email: String,
    pub remotes: Vec<remote::Remote>,
}

impl Config {
    pub fn new() -> io::Result<Config> {
        let mut name = String::new();
        let mut email = String::new();
        let mut remotes = Vec::new();
        let mut cur_section = String::new();

        let config_file = Path::new(".git").join("config");
        if config_file.exists() {
            let data = fs::read_to_string(config_file)?;
            for line in data.lines().map(|l| l.trim()) {
                if line.starts_with("[") && line.ends_with("]") {
                    cur_section = line.to_string();
                }

                let elem: Vec<&str> = line.split('=').collect();
                if elem.len() != 2 {
                    continue;
                }

                // [section]
                //      var = value
                let var = elem[0].trim().to_string();
                let value = elem[1].trim().to_string();
                if var == "name" {
                    name = value;
                } else if var == "email" {
                    email = value;
                } else if var == "url" {
                    let (_, remote_name) = parse_section_name(&cur_section);
                    remotes.push(remote::Remote {
                        name: remote_name.to_string(),
                        url: value,
                    });
                }
            }
        }

        Ok(Config {
            name: name,
            email: email,
            remotes: remotes,
        })
    }

    pub fn add(&mut self, section: &str, subsection: &str, value: &str) {
        match section {
            "user" => match subsection {
                "name" => self.name = value.to_string(),
                "email" => self.email = value.to_string(),
                sub => println!("config: unknown user subsection '{}'", sub),
            },
            "remote" => {
                self.remotes.push(remote::Remote {
                    name: subsection.to_string(),
                    url: value.to_string(),
                });
            }
            sct => println!("config: unknown section '{}'", sct),
        }
    }

    pub fn get(&self, section: &str, subsection: &str) -> Option<String> {
        match section {
            "user" => match subsection {
                "name" => return Some(self.name.to_string()),
                "email" => return Some(self.email.to_string()),
                sub => println!("config: unknown user subsection '{}'", sub),
            },
            "remote" => {
                for remote in &self.remotes {
                    if remote.name == subsection {
                        return Some(remote.url.to_string());
                    }
                }
                println!("config: unknown remote '{}'", subsection);
            }
            sct => println!("config: unknown section '{}'", sct),
        }

        None
    }

    pub fn unset(&mut self, section: &str, subsection: &str) {
        match section {
            "user" => match subsection {
                "name" => self.name = String::new(),
                "email" => self.email = String::new(),
                sub => println!("config: unknown user subsection '{}'", sub),
            },
            "remote" => self.remotes.retain(|r| r.name != subsection),
            sct => println!("config: unknown section '{}'", sct),
        }
    }

    pub fn write_config(&self) -> io::Result<()> {
        let mut config_fmt = format!("[user]\n\tname = {}\n\temail = {}\n", self.name, self.email);
        for remote in &self.remotes {
            let remote_entry = format!("[remote \"{}\"]\n\turl = {}\n", remote.name, remote.url);
            config_fmt = format!("{}{}", config_fmt, remote_entry);
        }

        let config_file = Path::new(".git").join("config");
        fs::write(config_file, config_fmt)?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.email.is_empty() && self.remotes.is_empty()
    }
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

pub fn config(option: &str, section: &str, value: &str) -> io::Result<()> {
    let mut user = Config::new()?;
    let (section, subsection) = parse_section_name(section);
    match option {
        "add" => user.add(&section, &subsection, &value),
        "get" => {
            if let Some(val) = user.get(&section, &subsection) {
                println!("{}", val);
            }
        }
        "unset" => user.unset(&section, &subsection),
        "list" => {
            println!("user.name = {}", user.name);
            println!("user.email = {}", user.email);
            for remote in &user.remotes {
                println!("remote.{} = {}", remote.name, remote.url);
            }
        }
        _ => unreachable!(),
    }

    let is_modif = option == "add" || option == "unset";
    if is_modif {
        user.write_config()?;
    }

    Ok(())
}

fn parse_section_name(section: &str) -> (String, String) {
    match section.starts_with("[") && section.ends_with("]") {
        // [section "subsection"] or [section]
        true => {
            let section = &section[1..section.len() - 1];
            let space_idx = match section.find(' ') {
                Some(i) => i,
                None => return (section.to_string(), String::new()),
            };
            let (section, subsection) = section.split_at(space_idx);
            let subsection = &subsection[2..subsection.len() - 1];
            return (section.to_string(), subsection.to_string());
        }

        // section.subsection or section
        false => {
            let dot_idx = match section.find('.') {
                Some(i) => i,
                None => return (section.to_string(), String::new()),
            };
            let (section, subsection) = section.split_at(dot_idx);
            let mut subsection = &subsection[1..];
            // If input is section.subsection.variable, ignore variable
            if let Some(dot_idx) = subsection.find('.') {
                subsection = &subsection[..dot_idx];
            }
            return (section.to_string(), subsection.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use builtin::config::parse_section_name;

    #[test]
    fn section_parser() {
        let exp = (String::from("user"), String::new());
        assert_eq!(exp, parse_section_name("[user]"));

        let exp = (String::from("user"), String::from("name"));
        assert_eq!(exp, parse_section_name("user.name"));

        let exp = (String::from("remote"), String::from("name"));
        assert_eq!(exp, parse_section_name("[remote \"name\"]"));

        let exp = (String::from("remote"), String::from("name"));
        assert_eq!(exp, parse_section_name("remote.name"));

        let exp = (String::from("remote"), String::from("name"));
        assert_eq!(exp, parse_section_name("remote.name.url"));

        let exp = (String::from("single"), String::new());
        assert_eq!(exp, parse_section_name("single"));
    }
}
