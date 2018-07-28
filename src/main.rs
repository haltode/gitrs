mod bits;
mod builtin;
mod cli;
mod environment;
mod index;
mod object;
mod refs;
mod sha1;
mod zlib;

use std::env;

use builtin::add;
use builtin::branch;
use builtin::cat_file;
use builtin::checkout;
use builtin::commit;
use builtin::config;
use builtin::diff;
use builtin::hash_object;
use builtin::init;
use builtin::ls_files;
use builtin::read_tree;
use builtin::status;
use builtin::write_tree;

fn main() {
    if !environment::is_inside_working_dir() {
        println!("Not a git repository (or any of the parent directories)");
        return;
    }

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        print_help();
        return;
    }

    let (args, flags) = cli::split_args_from_flags(args);
    let cmd = &args[1];
    match &cmd[..] {
        "init" => {
            let default_path = String::new();
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
                    cli::get_flag_value(&flags, "--type", "-t").unwrap_or(default_obj_type);

                let write = cli::has_flag(&flags, "--write", "-w");

                match hash_object::hash_object(data, &obj_type, write) {
                    Ok(hash) => println!("{}", hash),
                    Err(why) => println!("Cannot hash object: {:?}", why),
                }
            }
        }

        "cat-file" => {
            if args.len() == 2 || flags.len() == 0 {
                println!("cat-file: command takes 'hash' and 'mode' as arguments.");
            } else {
                let hash_prefix = &args[2];
                let mode = &flags[0];
                if let Err(why) = cat_file::cat_file(hash_prefix, mode) {
                    println!("Cannot retrieve object info: {:?}", why);
                }
            }
        }

        "ls-files" => {
            let stage = cli::has_flag(&flags, "--stage", "-s");
            ls_files::ls_files(stage);
        }

        "status" => {
            if let Err(why) = status::status() {
                println!("Could not retrieve status: {:?}", why);
            }
        }

        "diff" => {
            let paths = &args[2..];
            if let Err(why) = diff::diff(paths) {
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
                if let Err(why) = cat_file::cat_file(hash, "print") {
                    println!("Cannot retrieve object info: {:?}", why);
                }
            }
        }

        "commit" => {
            if args.len() == 2 {
                println!("commit: command takes a 'message' argument.");
            } else {
                let message = &args[2];
                if let Err(why) = commit::commit(message) {
                    println!("Could not commit: {:?}", why);
                }
            }
        }

        "config" => {
            if flags.len() == 0 {
                println!("config: command takes option such as '--add', '--list', etc.");
            } else {
                let default_val = String::new();
                let section = args.get(2).unwrap_or(&default_val);
                let value = args.get(3).unwrap_or(&default_val);
                let option = &flags[0][2..];

                if let Err(why) = config::config(option, section, value) {
                    println!("Could not use config file: {:?}", why);
                }
            }
        }

        "branch" => {
            let default_val = String::new();
            let name = args.get(2).unwrap_or(&default_val);
            let flag = flags.get(0).unwrap_or(&default_val);
            if let Err(why) = branch::branch(name, flag) {
                println!("Could not use branch: {:?}", why);
            }
        }

        "checkout" => {
            if args.len() == 2 {
                println!("checkout: command takes a 'ref' argument.");
            } else {
                let ref_name = &args[2];
                if let Err(why) = checkout::checkout(ref_name) {
                    println!("Could not checkout: {:?}", why);
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
