use std::io;
use std::str;

use builtin::status;
use object;
use object::Object;
use refs;
use working_dir;

#[derive(Debug)]
pub enum Error {
    AlreadyOnIt,
    ObjectError(object::Error),
    RefError(io::Error),
    ReferenceNotACommit,
    WorkingDirError(working_dir::Error),
    WorkingDirNotClean,
}

pub fn cmd_checkout(args: &[String]) {
    if args.is_empty() {
        println!("checkout: command takes a 'ref' argument.");
    } else {
        let ref_name = &args[0];
        if let Err(why) = checkout(ref_name) {
            println!("Could not checkout: {:?}", why);
        }
    }
}

fn checkout(ref_name: &str) -> Result<(), Error> {
    if !status::is_clean_working_dir() {
        return Err(Error::WorkingDirNotClean);
    }

    let will_detach_head = !refs::is_branch(&ref_name);
    let commit = match will_detach_head {
        true => ref_name.to_string(),
        false => refs::get_ref_hash(&ref_name).map_err(Error::RefError)?,
    };

    let object = Object::new(&commit).map_err(Error::ObjectError)?;
    if object.obj_type != "commit" {
        return Err(Error::ReferenceNotACommit);
    }

    let head = refs::get_ref_hash("HEAD").map_err(Error::RefError)?;
    if ref_name == head {
        return Err(Error::AlreadyOnIt);
    }

    working_dir::update_from_commit(&commit).map_err(Error::WorkingDirError)?;
    refs::write_to_ref("HEAD", ref_name).map_err(Error::RefError)?;

    if will_detach_head {
        println!("Note: checking out {}", ref_name);
        println!("You are in detached HEAD state.");
    } else {
        println!("Switched to branch {}", ref_name);
    }

    Ok(())
}
