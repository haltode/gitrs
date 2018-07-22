use std::str;

use object;
use read_tree;

pub fn cat_file(hash_prefix: &str, mode: &str) {
    let object = match object::get_object(hash_prefix) {
        Ok(obj) => obj,
        Err(why) => {
            println!("cat_file: error while parsing the object: {:?}", why);
            return;
        }
    };

    match mode {
        "type" => println!("{}", object.obj_type),
        "size" => println!("{}", object.obj_size),
        "print" => match object.obj_type.as_str() {
            "blob" | "commit" => {
                let data = str::from_utf8(&object.data).expect("invalid utf-8 in object data");
                println!("{}", data);
            }
            "tree" => {
                match read_tree::read_tree(hash_prefix) {
                    Ok(entries) => {
                        for entry in entries {
                            println!("{:o} {} {}", entry.mode, entry.hash, entry.path);
                        }
                    }
                    Err(why) => println!("cat_file: could not read tree object: {:?}", why),
                };
            }
            tp => println!("unknown object type: {}", tp),
        },
        fmt => println!("cat_file: unknown option mode: {}", fmt),
    }
}
