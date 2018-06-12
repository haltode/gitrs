mod init;
mod sha1;

fn main() {
    match init::init("test") {
        Ok(_) => {}
        Err(why) => println!("Could not initialize git repository: {:?}", why),
    };

    println!("hash: {}", sha1::sha1("abc"));
}