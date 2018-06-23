use std::fs;
use std::io;
use std::path::Path;

use sha1;
use zlib;

pub fn hash_object(data: &str, obj_type: &str, write: bool) -> Result<String, io::Error> {
    let header = format!("{} {}", obj_type, data.len());
    let data = format!("{}\x00{}", header, data);
    let hash = sha1::sha1(&data);

    if write {
        let dir = Path::new(".git").join("objects").join(&hash[..2]);
        let file = Path::new(&dir).join(&hash[2..]);

        if file.exists() {
            println!("hash_object: file already exists (ignoring write)");
        } else {
            fs::create_dir_all(&dir)?;
            fs::write(&file, zlib::compress(data.as_bytes().to_vec()))?;
        }
    }

    return Ok(hash);
}
