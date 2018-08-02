use std::str;

use builtin::read_tree;
use cli;
use object;
use object::Object;

#[derive(Debug)]
pub enum Error {
    ObjectError(object::Error),
    TreeError(read_tree::Error),
}

pub fn cmd_cat_file(args: &[String], flags: &[String]) {
    let accepted_flags = ["--type", "-t", "--size", "-s", "--print", "-p"];
    if cli::has_known_flags(flags, &accepted_flags) {
        if args.is_empty() || flags.is_empty() {
            println!("cat-file: command takes 'hash' and 'mode' as arguments.");
        } else {
            let hash_prefix = &args[0];
            let mode = &flags[0];
            if let Err(why) = cat_file(hash_prefix, mode) {
                println!("Cannot retrieve object info: {:?}", why);
            }
        }
    }
}

pub fn cat_file(hash_prefix: &str, mode: &str) -> Result<(), Error> {
    let object = Object::new(hash_prefix).map_err(Error::ObjectError)?;
    match mode {
        "--type" | "-t" => println!("{}", object.obj_type),
        "--size" | "-s" => println!("{}", object.obj_size),
        "--print" | "-p" => match object.obj_type.as_str() {
            "blob" | "commit" => {
                let data = str::from_utf8(&object.data).unwrap();
                println!("{}", data);
            }
            "tree" => {
                let entries = read_tree::read_tree(hash_prefix).map_err(Error::TreeError)?;
                for entry in entries {
                    println!("{:o} {} {}", entry.mode, entry.hash, entry.path);
                }
            }
            tp => println!("unknown object type: {}", tp),
        },
        _ => unreachable!(),
    }

    Ok(())
}
