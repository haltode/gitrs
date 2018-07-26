mod add;
mod cat_file;
mod commit;
mod config;
mod diff;
mod hash_object;
mod index;
mod init;
mod ls_files;
mod object;
mod read_tree;
mod status;
mod utils;
mod write_tree;
mod zlib;

use std::env;

use utils::cli;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        print_help();
        return;
    }

    let cmd = &args[1];
    match &cmd[..] {
        "init" => {
            let default_path = String::from("");
            let path = args.get(2).unwrap_or(&default_path);
            if let Err(why) = init::init(path) {
                println!("Could not initialize git repository: {:?}", why);
            }
        }

        "hash-object" => {
            if args.len() == 2 {
                println!("hash-object: command takes a 'data' argument.");
            } else {
                let data = &args[2].as_bytes();

                let default_obj_type = String::from("blob");
                let obj_type =
                    cli::get_flag_value(&args, "--type", "-t").unwrap_or(default_obj_type);

                let write = cli::has_flag(&args, "--write", "-w");

                match hash_object::hash_object(data, &obj_type, write) {
                    Ok(hash) => println!("{}", hash),
                    Err(why) => println!("Cannot hash object: {}", why),
                }
            }
        }

        "cat-file" => {
            if args.len() <= 3 {
                println!("cat-file: command takes 'hash' and 'mode' as arguments.");
            } else {
                let hash_prefix = &args[2];
                if cli::has_flag(&args, "--type", "-t") {
                    cat_file::cat_file(hash_prefix, "type");
                } else if cli::has_flag(&args, "--size", "-s") {
                    cat_file::cat_file(hash_prefix, "size");
                } else if cli::has_flag(&args, "--print", "-p") {
                    cat_file::cat_file(hash_prefix, "print");
                } else {
                    println!("cat-file: unknown 'mode' option.");
                }
            }
        }

        "ls-files" => {
            let stage = cli::has_flag(&args, "--stage", "-s");
            ls_files::ls_files(stage);
        }

        "status" => {
            if let Err(why) = status::status() {
                println!("Could not retrieve status: {:?}", why);
            }
        }

        "diff" => {
            if let Err(why) = diff::diff() {
                println!("Could not show diff: {:?}", why);
            }
        }

        "add" => {
            if args.len() == 2 {
                println!("add: command takes paths as arguments.");
            } else {
                let paths = &args[2..];
                if let Err(why) = add::add(paths) {
                    println!("Could not add paths: {:?}", why);
                }
            }
        }

        "write-tree" => match write_tree::write_tree() {
            Ok(hash) => println!("{}", hash),
            Err(why) => println!("Could not create tree object: {:?}", why),
        },

        "read-tree" => {
            if args.len() == 2 {
                println!("read-tree: command takes a 'hash' argument.");
            } else {
                let hash = &args[2];
                cat_file::cat_file(hash, "print");
            }
        }

        "commit" => {
            if args.len() == 2 {
                println!("commit: command takes a 'message' argument.");
            } else {
                let message = &args[2];
                match commit::commit(message) {
                    Ok(hash) => println!("commit on master: {}", hash),
                    Err(why) => println!("Could not commit: {:?}", why),
                }
            }
        }

        "config" => {
            if args.len() == 2 {
                println!("config: command takes option such as '--add', '--list', etc.");
            } else {
                let option = &args[2][2..];
                let section = match args.get(3) {
                    Some(s) => s,
                    None => "",
                };
                let value = match args.get(4) {
                    Some(s) => s,
                    None => "",
                };

                if let Err(why) = config::config(option, section, value) {
                    println!("Could not use config file: {:?}", why);
                }
            }
        }

        "help" | _ => {
            print_help();
        }
    }
}

fn print_help() {
    println!("TODO: write help!");
}
