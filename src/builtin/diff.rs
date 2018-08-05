use std::cmp;
use std::fs;
use std::io;
use std::str;

use index;
use object;
use object::Object;

#[derive(Debug)]
pub enum Error {
    IndexError(index::Error),
    IoError(io::Error),
    ObjectError(object::Error),
}

enum State {
    Ins,
    Del,
    Eq,
}

pub fn cmd_diff(args: &[String]) {
    if let Err(why) = diff(args) {
        println!("Could not show diff: {:?}", why);
    }
}

fn diff(paths: &[String]) -> Result<(), Error> {
    let entries = index::read_entries().map_err(Error::IndexError)?;
    for entry in &entries {
        let path = &entry.path;
        if !paths.is_empty() && !paths.contains(path) {
            continue;
        }

        let object = Object::new(&entry.hash).map_err(Error::ObjectError)?;
        if object.obj_type != "blob" {
            continue;
        }

        let stored_data = str::from_utf8(&object.data).unwrap();
        let actual_data = fs::read_to_string(path).map_err(Error::IoError)?;

        let stored_lines: Vec<&str> = stored_data.split('\n').collect();
        let actual_lines: Vec<&str> = actual_data.split('\n').collect();
        if stored_lines == actual_lines {
            continue;
        }

        println!("{}:", path);
        for (state, line) in lcs_diff(&stored_lines, &actual_lines) {
            let c = match state {
                State::Ins => '+',
                State::Del => '-',
                State::Eq => ' ',
            };
            println!("{}{}", c, line);
        }
    }

    Ok(())
}

fn lcs_diff(a: &[&str], b: &[&str]) -> Vec<(State, String)> {
    let mut res = Vec::new();
    let lcs = longest_common_subseq(a, b);
    let mut i = a.len();
    let mut j = b.len();
    loop {
        if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
            res.push((State::Eq, a[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            res.push((State::Ins, b[j - 1].to_string()));
            j -= 1;
        } else if i > 0 && (j == 0 || lcs[i][j - 1] < lcs[i - 1][j]) {
            res.push((State::Del, a[i - 1].to_string()));
            i -= 1;
        } else {
            break;
        }
    }
    res.reverse();
    return res;
}

fn longest_common_subseq(a: &[&str], b: &[&str]) -> Vec<Vec<u32>> {
    let m = a.len() + 1;
    let n = b.len() + 1;
    let mut lcs = vec![vec![0u32; n]; m];
    for i in 1..m {
        for j in 1..n {
            if a[i - 1] == b[j - 1] {
                lcs[i][j] = lcs[i - 1][j - 1] + 1;
            } else {
                lcs[i][j] = cmp::max(lcs[i][j - 1], lcs[i - 1][j]);
            }
        }
    }
    return lcs;
}
