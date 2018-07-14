use std::fs;
use std::io;
use std::path::Path;

use utils::sha1;
use zlib;

pub fn hash_object(data: &str, obj_type: &str, write: bool) -> io::Result<String> {
    let header = format!("{} {}", obj_type, data.len());
    let data = format!("{}\x00{}", header, data);
    let hash = sha1::sha1_str(&data);

    if write {
        let dir = Path::new(".git").join("objects").join(&hash[..2]);
        let file = Path::new(&dir).join(&hash[2..]);

        if file.exists() {
            println!("hash_object: file already exists (ignoring write)");
        } else {
            fs::create_dir_all(&dir)?;
            let compressed_data = zlib::compress(data.as_bytes().to_vec());
            fs::write(&file, &compressed_data)?;
        }
    }

    Ok(hash)
}
