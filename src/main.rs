mod init;

fn main() {
    match init::init("test") {
        Ok(_) => {}
        Err(why) => println!("Could not initialize git repository: {:?}", why),
    };
}
