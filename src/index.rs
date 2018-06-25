// * git/Documentation/technical/index-format.txt
//   https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

use std::fs;
use std::path::Path;
use std::str;

use bits::big_endian;
use sha1;

pub struct Entry {
    ctime_sec: u32,
    ctime_nan: u32,
    mtime_sec: u32,
    mtime_nan: u32,
    dev: u32,
    ino: u32,
    pub mode: u32,
    uid: u32,
    gid: u32,
    size: u32,
    pub hash: String,
    pub flags: u16,
    pub path: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidHeaderSignature,
    InvalidIndexVersion,
}

pub fn get_entries() -> Result<Vec<Entry>, Error> {
    let index_path = Path::new(".git").join("index");
    let bytes = fs::read(index_path).expect("cannot read index");
    let signature = str::from_utf8(&bytes[0..4]).expect("invalid utf-8 in index signature");
    if signature != "DIRC" {
        return Err(Error::InvalidHeaderSignature);
    }

    let version = big_endian::u8_slice_to_u32(&bytes[4..]);
    if version != 2 {
        return Err(Error::InvalidIndexVersion);
    }

    let nb_entries = big_endian::u8_slice_to_u32(&bytes[8..]) as usize;
    let mut entries = Vec::new();
    let mut idx = 12;
    for _ in 0..nb_entries {
        let mut fields = [0u32; 10];
        for e in 0..10 {
            fields[e] = big_endian::u8_slice_to_u32(&bytes[idx..]);
            idx += 4;
        }

        let hash = sha1::u8_slice_hash_to_hex_str(&bytes[idx..]);
        idx += 20;

        let flags = big_endian::u8_slice_to_u16(&bytes[idx..]);
        idx += 2;

        let null_idx = bytes[idx..]
            .iter()
            .position(|&x| x == 0)
            .expect("index entry does not terminate by null byte");
        let path = str::from_utf8(&bytes[idx..idx + null_idx])
            .expect("invalid utf-8 in index entry path")
            .to_string();
        idx += null_idx;

        let entry_len = 62 + path.len();
        let padding = ((entry_len + 8) / 8) * 8 - entry_len;
        idx += padding;

        entries.push(Entry {
            ctime_sec: fields[0],
            ctime_nan: fields[1],
            mtime_sec: fields[2],
            mtime_nan: fields[3],
            dev: fields[4],
            ino: fields[5],
            mode: fields[6],
            uid: fields[7],
            gid: fields[8],
            size: fields[9],
            hash: hash,
            flags: flags,
            path: path,
        });
    }

    // TODO: checksum

    Ok(entries)
}
