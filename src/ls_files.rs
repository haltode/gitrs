use index;

pub fn ls_files(stage: bool) {
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
