use std::fs;
use std::io;
use std::path::Path;

use cli;
use sha1;
use zlib;

pub fn cmd_hash_object(args: &[String], flags: &[String]) {
    let accepted_flags = ["--type", "-t", "--write", "-w"];
    if cli::has_known_flags(flags, &accepted_flags) {
        if args.is_empty() {
            println!("hash-object: command takes a 'data' argument.");
        } else {
            let data = &args[0].as_bytes();
            let obj_type = match cli::has_flag(flags, "--type", "-t") {
                true => match &args.get(1) {
                    Some(t) => t,
                    None => {
                        println!("hash-object: missing 'type' argument.");
                        return;
                    }
                },
                false => "blob",
            };
            let write = cli::has_flag(&flags, "--write", "-w");

            match hash_object(data, &obj_type, write) {
                Ok(hash) => println!("{}", hash),
                Err(why) => println!("Cannot hash object: {:?}", why),
            }
        }
    }
}

pub fn hash_object(data: &[u8], obj_type: &str, write: bool) -> io::Result<String> {
    let header = format!("{} {}", obj_type, data.len());
    let mut full_data = Vec::new();
    full_data.extend(header.as_bytes());
    full_data.push(0);
    full_data.extend(data);

    let hash = sha1::sha1(&full_data);

    if write {
        let obj_dir = Path::new(".git").join("objects").join(&hash[..2]);
        let obj_path = Path::new(&obj_dir).join(&hash[2..]);

        if !obj_path.exists() {
            fs::create_dir_all(&obj_dir)?;
            let compressed_data = zlib::compress(full_data);
            fs::write(&obj_path, &compressed_data)?;
        }
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use builtin::hash_object::hash_object;

    #[test]
    fn short() {
        let res = hash_object("this is a test!".as_bytes(), "blob", false).unwrap();
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
             yuPeiOI1ooBwlNDLqqFxUVzfHeVpVila3PyrMrMSMq0CV"
                .as_bytes(),
            "blob",
            false,
        ).unwrap();
        assert_eq!("9a1be2ae6deb625c3e4d821f56016ee582d45fa0", &res);
    }

    #[test]
    fn multiline() {
        let res = hash_object(
            "This is a multi-line string litteral
used as a test file sample!\n"
                .as_bytes(),
            "blob",
            false,
        ).unwrap();
        assert_eq!("ca1bd6f977c9c4319096dde65ab7824d6d249d12", &res);
    }
}
