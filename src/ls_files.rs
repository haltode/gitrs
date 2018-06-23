use index;

pub fn ls_files(stage: bool) {
    for entry in index::get_entries() {
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
