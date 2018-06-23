mod bits;
mod cat_file;
mod cli;
mod hash_object;
mod index;
mod init;
mod object;
mod sha1;
mod zlib;

use std::env;

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

        "hash_object" => {
            if args.len() == 2 {
                println!("hash_object: command takes a 'data' argument.");
            } else {
                let data = &args[2];

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

        "cat_file" => {
            if args.len() <= 3 {
                println!("cat_file: command takes 'hash' and 'mode' as arguments.");
            } else {
                let hash_prefix = &args[2];
                if cli::has_flag(&args, "--type", "-t") {
                    cat_file::cat_file(hash_prefix, "type");
                } else if cli::has_flag(&args, "--size", "-s") {
                    cat_file::cat_file(hash_prefix, "size");
                } else if cli::has_flag(&args, "--print", "-p") {
                    cat_file::cat_file(hash_prefix, "print");
                } else {
                    println!("cat_file: unknown 'mode' option.");
                }
            }
        }

        "ls_files" => {
            let stage = cli::has_flag(&args, "--stage", "-s");
            index::ls_files(stage);
        }

        "status" => {
            if let Err(why) = index::status() {
                println!("Could not retrieve status: {}", why);
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
