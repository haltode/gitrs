use std::fs;
use std::io;
use std::path::Path;

pub fn read_ref(name: &str) -> io::Result<String> {
    let ref_name = full_ref_name(name);
    let ref_path = Path::new(".git").join(ref_name);

    let mut value = fs::read_to_string(ref_path)?;
    // Remove '\n' character
    value.pop();

    let head_prefix = "ref: refs/heads/";
    if value.starts_with(&head_prefix) {
        value = value.split_off(head_prefix.len());
    }

    let fetch_mark = " branch ";
    if value.contains(fetch_mark) {
        // Only retrieve the hash
        value.split_off(40);
    }

    Ok(value)
}

pub fn get_ref_hash(name: &str) -> io::Result<String> {
    let value = read_ref(name)?;
    if name == "HEAD" && is_branch(&value) {
        return read_ref(&value);
    }

    Ok(value)
}

pub fn write_to_ref(name: &str, value: &str) -> io::Result<()> {
    let ref_name = full_ref_name(name);
    let ref_path = Path::new(".git").join(ref_name);

    let prefix = match name == "HEAD" && is_branch(&value) {
        true => "ref: refs/heads/",
        false => "",
    };
    let formated_value = format!("{}{}\n", prefix, value);

    fs::write(ref_path, formated_value)?;
    Ok(())
}

pub fn exists_ref(name: &str) -> bool {
    let ref_name = full_ref_name(name);
    Path::new(".git").join(ref_name).exists() || is_detached_head()
}

pub fn is_branch(name: &str) -> bool {
    Path::new(".git")
        .join("refs")
        .join("heads")
        .join(&name)
        .exists()
}

pub fn is_detached_head() -> bool {
    let head_path = Path::new(".git").join("HEAD");
    let head = match fs::read_to_string(head_path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    !head.starts_with("ref: refs/heads/")
}

fn full_ref_name(name: &str) -> String {
    if name == "HEAD" || name == "FETCH_HEAD" || name.starts_with("refs/heads/") {
        name.to_string()
    } else {
        format!("refs/heads/{}", name)
    }
}
