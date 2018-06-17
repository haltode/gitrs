use object;

pub fn cat_file(hash_prefix: &str, mode: &str) {
    let object = match object::parse(hash_prefix) {
        Ok(obj) => obj,
        Err(why) => {
            println!("cat_file: error while parsing the object: {:?}", why);
            return;
        }
    };

    match mode {
        "type" => println!("{}", object.obj_type),
        "size" => println!("{}", object.obj_size),
        "print" => println!("{}", object.data),
        fmt => println!("cat_file: unknown option mode: {}", fmt),
    }
}
