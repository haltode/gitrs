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

#[cfg(test)]
mod tests {
    use hash_object::hash_object;

    #[test]
    fn short() {
        let res = hash_object("this is a test!", "blob", false).unwrap();
        assert_eq!("ca8d93e91ccd585c740d9a483ab11c428eb085f2", &res);
    }

    #[test]
    fn long() {
        let res = hash_object(
            "i3Kyc3VSFY359Szkg9q0thD6aR5pfmn7z2gWexqC0KwM8odmUUei4qFFbMnbm4\
             yhUt5oBWvHv5DoeEvivf2Fq4hp5YsvdccpyyPErD9OPfQBHdrOF6DuMMRXwDuIS\
             bKMMoFYm3XOsA8za5YA5rqV4Pm699eSPYt15ymlRpJ3VPJhUBr34qRZRTxX4Q3m\
             ro7mUXVJPwwJJv44Svs3BwxGsA3EQuddSz1kKEp7JJWWUzBVFaKdTE79soRZr9i\
             eg5976Y1QEZ901aUaO0zfd8V09dWhvM53W0jyMkw4DlLBZlPXXKGjrVCmqwFJ7O\
             2LyVtiewFZ2uS8YHdftRr2eiVIisVrGoenqpXKBUkng32L3WSJ3Gs6chIshWbKp\
             bLhXmM7JjsioxqtnfA8Vwmhr3IGIYLSyOROv9JPMiwnaBdYpFwGc585ZaEKY1qe\
             IzQbHAleXzh9bMBPG91gkgs4jcNJvRlnzPNZ74fVFKmnf29Ue4UcUNVKDe2cVPq\
             nEmGhLcj6BzJ61CxiJUC4sZqhTtYZGbkV4Rb8FgwVRQUqRq5Zsb1Kh8eYbcyHRV\
             N3ih6wJWruBxixGAMLIseURVEUBnRc3nkYCMVsgkwRVevo8Ehp60Ih7eF4sarMX\
             6EH8caKYIv5A3SE6Owb6dQqYrbOL7EgXNOnCIwQxhz0aw2p4AYmHC22so8rfGbN\
             C1I95RXd9g38Xg4fm8AJORNGsEx0mVLy6GFLuiZ6KxNXci6wPg2BnZj5Pwg4ywT\
             yuPeiOI1ooBwlNDLqqFxUVzfHeVpVila3PyrMrMSMq0CV",
            "blob",
            false,
        ).unwrap();
        assert_eq!("9a1be2ae6deb625c3e4d821f56016ee582d45fa0", &res);
    }

    #[test]
    fn multiline() {
        let res = hash_object(
            "This is a multi-line string litteral
used as a test file sample!\n",
            "blob",
            false,
        ).unwrap();
        assert_eq!("ca1bd6f977c9c4319096dde65ab7824d6d249d12", &res);
    }
}
