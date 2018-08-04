use builtin::fetch;
use builtin::merge;

#[derive(Debug)]
pub enum Error {
    FetchError(fetch::Error),
    MergeError(merge::Error),
}

pub fn cmd_pull(args: &[String]) {
    if args.len() < 2 {
        println!("pull: takes 'remote' and 'branch' arguments");
    } else {
        let remote = &args[0];
        let branch = &args[1];
        if let Err(why) = pull(&remote, &branch) {
            println!("Could not pull: {:?}", why);
        }
    }
}

pub fn pull(remote: &str, branch: &str) -> Result<(), Error> {
    fetch::fetch(&remote, &branch).map_err(Error::FetchError)?;
    merge::merge("FETCH_HEAD").map_err(Error::MergeError)?;
    Ok(())
}
