pub fn has_flag(args: &[String], long_fmt: &str, short_fmt: &str) -> bool {
    args.iter().any(|x| x == long_fmt || x == short_fmt)
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
