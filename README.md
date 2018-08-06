# gitrs

A small re-implementation of [git](https://git-scm.com/) (a distributed version
control system) written in [Rust](https://www.rust-lang.org/):

- basic commands: `init`, `config`, `add`, `commit`, `status`, `diff`, `log`.
- branches: `branch`, `checkout`, `merge`.
- remotes: `clone`, `fetch`, `push`, `pull`, `remote`.
- plumbing: `hash-object`, `cat-file`, `ls-files`, `read-tree`, `write-tree`.

### Building it

```bash
$ cargo build --release
$ cd target/release
$ ./gitrs
```

### Running it

```bash
$ mkdir repo
$ cd repo
$ gitrs init
Initialized empty Git repository in .git
$ gitrs config --add user.name "John Doe"
$ gitrs config --add user.email "john.doe@something.com"

$ echo 'Hello world!' > file_a
$ gitrs status
new: file_a
$ gitrs add file_a
$ gitrs commit -m "first commit"
[master 5b0cd52] first commit

$ cd ..
$ gitrs clone repo copy
Initialized empty Git repository in copy/.git
Count: 3 objects
From: /home/haltode/repo
Fast-forward
Cloning into copy
$ cp repo/.git/config copy/.git/config
$ cd copy
$ echo 'new file' > file_b
$ gitrs add file_b
$ gitrs commit -m "second commit"
[master 3940393] second commit

$ cd ../repo
$ gitrs remote add copy_remote ../copy
$ gitrs branch new_b
$ gitrs pull copy_remote master
Count: 3 objects
From: ../copy
Fast-forward

$ gitrs checkout new_b
Switched to branch new_b
$ echo 'new line' >> file_a
$ gitrs status
modified: file_a
$ gitrs diff
file_a:
 Hello world!
+new_line
$ gitrs add file_a
$ gitrs commit -m "third commit"
[new_b 9e4e36b] third commit
$ gitrs checkout master
Switched to branch master
$ gitrs merge new_b
Merge new_b into master
[master 7b8e051] Merge new_b into master
```

## Resources used

- [Pro Git book](https://git-scm.com/book/en/v2)
- [Git docs](https://git-scm.com/docs)
- [gitcore-tutorial](https://git-scm.com/docs/gitcore-tutorial)
- [gitrepository-layout](https://git-scm.com/docs/gitrepository-layout)
- [Git user manual](https://git-scm.com/docs/user-manual.html)
- [Git from the bottom up](https://jwiegley.github.io/git-from-the-bottom-up/)
- [The curious coder’s guide to git](https://matthew-brett.github.io/curious-git/)

## Why?

I wanted a fun project to learn more about the Rust programming language and git
inner workings at the same time. The sole purpose of this project is
educational. As a challenge I also restricted myself to the Rust standard
library, thus re-implementing everything else that I might need such as: sha-1
hash function, zlib compress/decompress functions, etc. This is absurd and
definitely not good practice, but again the only aim was to learn, so every
opportunity is a great excuse to code in Rust!
