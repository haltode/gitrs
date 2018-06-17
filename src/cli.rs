pub fn has_flag(args: &Vec<String>, long_fmt: &str, short_fmt: &str) -> bool {
    args.iter().any(|x| x == long_fmt || x == short_fmt)
}

pub fn get_flag_value(args: &Vec<String>, long_fmt: &str, short_fmt: &str) -> Option<String> {
    let pos = args.iter().position(|x| x == long_fmt || x == short_fmt);
    match pos {
        None => None,
        Some(x) if (x + 1) >= args.len() => {
            println!("warning: missing flag value (ignoring option).");
            return None;
        }
        Some(x) => {
            return Some(args[x + 1].to_string());
        }
    }
}
