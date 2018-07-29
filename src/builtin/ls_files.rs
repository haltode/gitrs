use cli;
use index;

pub fn cmd_ls_files(flags: &[String]) {
    let accepted_flags = ["--stage", "-s"];
    if cli::has_known_flags(flags, &accepted_flags) {
        let stage = cli::has_flag(&flags, "--stage", "-s");
        if let Err(why) = ls_files(stage) {
            println!("Could not print index files: {:?}", why);
        }
    }
}

fn ls_files(stage: bool) -> Result<(), index::Error> {
    let entries = index::read_entries()?;
    for entry in entries {
        if stage {
            let stage_nb = (entry.flags >> 12) & 3;
            println!(
                "{:6o} {} {}\t{}",
                entry.mode, entry.hash, stage_nb, entry.path
            );
        } else {
            println!("{}", entry.path);
        }
    }

    Ok(())
}
