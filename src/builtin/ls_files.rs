use cli;
use index;

pub fn cmd_ls_files(flags: &[String]) {
    let accepted_flags = ["--stage", "-s"];
    if cli::has_known_flags(flags, &accepted_flags) {
        let stage = cli::has_flag(&flags, "--stage", "-s");
        ls_files(stage);
    }
}

fn ls_files(stage: bool) {
    let entries = match index::read_entries() {
        Ok(e) => e,
        Err(why) => {
            println!("ls-files: error while reading git index: {:?}", why);
            return;
        }
    };

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
}
