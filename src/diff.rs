// * diff
//   https://en.wikipedia.org/wiki/Diff
// * Longest Common Subsequence Problem
//   https://en.wikipedia.org/wiki/Longest_common_subsequence_problem
// * Myers algorithm
// * Patience algorithm

use std::cmp;
use std::fs;
use std::io;
use std::str;

use index;
use object;

#[derive(Debug)]
pub enum Error {
    IndexError(index::Error),
    IoError(io::Error),
    ObjectError(object::Error),
    Utf8Error(str::Utf8Error),
}

enum State {
    Ins,
    Del,
    Eq,
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

fn lcs_diff(a: &[&str], b: &[&str]) -> Vec<(State, String)> {
    let mut res = Vec::new();
    let lcs = longest_common_subseq(a, b);
    let mut i = a.len() - 1;
    let mut j = b.len() - 1;
    while i != 0 || j != 0 {
        if i > 0 && j > 0 && a[i] == b[j] {
            res.push((State::Eq, a[i].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            res.push((State::Ins, b[j].to_string()));
            j -= 1;
        } else if i > 0 && (j == 0 || lcs[i][j - 1] < lcs[i - 1][j]) {
            res.push((State::Del, a[i].to_string()));
            i -= 1;
        }
    }
    res.reverse();
    return res;
}

// TODO: add context format
// TODO: take vec of files as parameters
pub fn diff() -> Result<(), Error> {
    let entries = index::read_entries().map_err(Error::IndexError)?;
    for entry in &entries {
        let path = &entry.path;
        let hash = &entry.hash;
        let obj = object::parse(hash).map_err(Error::ObjectError)?;
        if obj.obj_type != "blob" {
            continue;
        }

        let stored_data = str::from_utf8(&obj.data).map_err(Error::Utf8Error)?;
        let actual_data = fs::read_to_string(path).map_err(Error::IoError)?;

        let stored_lines: Vec<&str> = stored_data.split("\n").collect();
        let actual_lines: Vec<&str> = actual_data.split("\n").collect();
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
