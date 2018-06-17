mod cli;
mod hash_object;
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

        "help" | _ => {
            print_help();
        }
    }
}

fn print_help() {
    println!("TODO: write help!");
}
