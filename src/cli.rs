pub fn has_flag(args: &[String], long_fmt: &str, short_fmt: &str) -> bool {
    args.iter().any(|x| x == long_fmt || x == short_fmt)
}

pub fn get_flag_value(args: &[String], long_fmt: &str, short_fmt: &str) -> Option<String> {
    let pos = args.iter().position(|x| x == long_fmt || x == short_fmt);
    match pos {
        None => None,
        Some(x) if (x + 1) >= args.len() => {
            println!("warning: missing flag value (ignoring option).");
            None
        }
        Some(x) => Some(args[x + 1].to_string()),
    }
}

pub fn has_known_flags(flags: &[String], known_flags: &[&str]) -> bool {
    for flag in flags {
        let is_known = known_flags.contains(&flag.as_str());
        if !is_known {
            println!("unknown flag: {}", flag);
            return false;
        }
    }
    return true;
}

pub fn split_args_from_flags(input: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut args = Vec::new();
    let mut flags = Vec::new();
    for opt in input {
        if opt.starts_with("-") {
            flags.push(opt);
        } else {
            args.push(opt);
        }
    }
    (args, flags)
}
