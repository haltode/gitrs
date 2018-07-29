mod bits;
mod builtin;
mod cli;
mod environment;
mod index;
mod object;
mod refs;
mod sha1;
mod zlib;

use std::env;

fn main() {
    if environment::get_working_dir().is_err() {
        println!("Not a git repository (or any of the parent directories)");
        return;
    }

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        print_help();
        return;
    }

    let (args, flags) = cli::split_args_from_flags(args);
    let cmd = &args[1];
    let args = &args[2..];
    match cmd.as_str() {
        "init" => builtin::init::cmd_init(&args),
        "hash-object" => builtin::hash_object::cmd_hash_object(&args, &flags),
        "cat-file" => builtin::cat_file::cmd_cat_file(&args, &flags),
        "ls-files" => builtin::ls_files::cmd_ls_files(&flags),
        "status" => builtin::status::cmd_status(),
        "diff" => builtin::diff::cmd_diff(&args),
        "add" => builtin::add::cmd_add(&args),
        "write-tree" => builtin::write_tree::cmd_write_tree(),
        "read-tree" => builtin::read_tree::cmd_read_tree(&args),
        "commit" => builtin::commit::cmd_commit(&args, &flags),
        "config" => builtin::config::cmd_config(&args, &flags),
        "branch" => builtin::branch::cmd_branch(&args, &flags),
        "checkout" => builtin::checkout::cmd_checkout(&args),
        "help" | _ => print_help(),
    }
}

fn print_help() {
    println!("TODO: write help!");
}
