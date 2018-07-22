// * git/Documentation/technical/index-format.txt
//   https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

use std::fs;
use std::io;
use std::path::Path;
use std::str;

use utils::bits::big_endian;
use utils::sha1;

#[derive(Debug)]
pub struct Entry {
    pub ctime_sec: u32,
    pub ctime_nan: u32,
    pub mtime_sec: u32,
    pub mtime_nan: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub hash: String,
    pub flags: u16,
    pub path: String,
}

#[derive(Debug)]
pub enum Error {
    EntryMissingNullByteEnding,
    InvalidChecksum,
    InvalidHash,
    InvalidHeaderSignature,
    InvalidIndexVersion,
    IoError(io::Error),
    Utf8Error(str::Utf8Error),
}

pub fn read_entries() -> Result<Vec<Entry>, Error> {
    let index = Path::new(".git").join("index");
    if !index.exists() {
        return Ok(vec![]);
    }

    let bytes = fs::read(index).map_err(Error::IoError)?;
    let signature = str::from_utf8(&bytes[0..4]).map_err(Error::Utf8Error)?;
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

        let null_idx = match bytes[idx..].iter().position(|&x| x == 0) {
            Some(i) => i,
            None => {
                return Err(Error::EntryMissingNullByteEnding);
            }
        };
        let path = str::from_utf8(&bytes[idx..idx + null_idx])
            .map_err(Error::Utf8Error)?
            .to_string();
        idx += null_idx;

        let entry_len = 62 + path.len();
        let padding_len = ((entry_len + 8) / 8) * 8 - entry_len;
        idx += padding_len;

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

    let checksum = sha1::u8_slice_hash_to_hex_str(&bytes[idx..]);
    let actual_hash = sha1::sha1(&bytes[..idx]);
    if actual_hash != checksum {
        return Err(Error::InvalidChecksum);
    }

    Ok(entries)
}

pub fn write_entries(entries: Vec<Entry>) -> Result<(), Error> {
    let mut compressed_entries = Vec::new();
    for entry in &entries {
        let fields = vec![
            entry.ctime_sec,
            entry.ctime_nan,
            entry.mtime_sec,
            entry.mtime_nan,
            entry.dev,
            entry.ino,
            entry.mode,
            entry.uid,
            entry.gid,
            entry.size,
        ];

        let mut bytes_entry = Vec::new();
        for field in fields {
            bytes_entry.extend(&big_endian::u32_to_u8(field));
        }

        let compressed_hash = match sha1::compress_hash(&entry.hash) {
            Some(hash) => hash,
            None => {
                return Err(Error::InvalidHash);
            }
        };
        bytes_entry.extend(&compressed_hash);
        bytes_entry.extend(&big_endian::u16_to_u8(entry.flags));
        bytes_entry.extend(entry.path.as_bytes());

        let entry_len = 62 + entry.path.len();
        let padding_len = ((entry_len + 8) / 8) * 8 - entry_len;
        let padding = vec![0u8; padding_len];
        bytes_entry.extend(&padding);

        compressed_entries.extend(&bytes_entry);
    }

    let mut header = vec![68, 73, 82, 67, 0, 0, 0, 2];
    header.extend(&big_endian::u32_to_u8(entries.len() as u32));

    let mut data = Vec::new();
    data.extend(&header);
    data.extend(&compressed_entries);

    let checksum = sha1::sha1(&data);
    let compressed_hash = match sha1::compress_hash(&checksum) {
        Some(hash) => hash,
        None => {
            return Err(Error::InvalidHash);
        }
    };
    data.extend(&compressed_hash);

    let index = Path::new(".git").join("index");
    fs::write(index, &data).map_err(Error::IoError)?;

    Ok(())
}
