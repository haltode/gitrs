use std::str;

use object;
use read_tree;

#[derive(Debug)]
pub enum Error {
    ObjectError(object::Error),
    TreeError(read_tree::Error),
    Utf8Error(str::Utf8Error),
}

pub fn cat_file(hash_prefix: &str, mode: &str) -> Result<(), Error> {
    let object = object::get_object(hash_prefix).map_err(Error::ObjectError)?;

    match mode {
        "--type" | "-t" | "type" => println!("{}", object.obj_type),
        "--size" | "-s" | "size" => println!("{}", object.obj_size),
        "--print" | "-p" | "print" => match object.obj_type.as_str() {
            "blob" | "commit" => {
                let data = str::from_utf8(&object.data).map_err(Error::Utf8Error)?;
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
        fmt => println!("cat-file: unknown option mode: {}", fmt),
    }

    Ok(())
}
