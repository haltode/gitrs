mod init;
mod sha1;
mod decompress;

use std::fs::File;
use std::io::Read;

fn main() {
    match init::init("test") {
        Ok(_) => {}
        Err(why) => println!("Could not initialize git repository: {:?}", why),
    };

    let file_name =
        "/home/haltode/projects/gitrs/.git/objects/5b/2d31aeb49be35a7a61446dfe259c69f1e5ca06";
    let mut f = File::open(file_name).unwrap();

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    println!("{:?}", decompress::decompress(buffer));
}
