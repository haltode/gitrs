use index;

pub fn add(paths: &[String]) -> Result<(), index::Error> {
    let mut entries = index::read_entries()?;
    entries.retain(|e| !paths.contains(&e.path));

    for path in paths {
        let entry = index::new_entry(path)?;
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    index::write_entries(entries)?;
    Ok(())
}
