use index;

pub fn cmd_add(args: &[String]) {
    if let Err(why) = add(args) {
        println!("Could not add paths: {:?}", why);
    }
}

fn add(paths: &[String]) -> Result<(), index::Error> {
    let mut entries = index::read_entries()?;
    entries.retain(|e| !paths.contains(&e.path));

    for path in paths {
        let entry = index::Entry::new(path)?;
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    index::write_entries(entries)?;
    Ok(())
}
