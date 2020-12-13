use crate::linq::collectors::IntoVec;
use std::ops::{Index, Range};
use std::slice::Iter;
use std::iter::{Peekable, Fuse};

pub struct Args {
    args: Vec<(String, usize)>,
    cmdline: String
}

pub struct ArgsIter<'a> {
    iter: Peekable<Fuse<Iter<'a, (String, usize)>>>,
    index: usize,
}

impl ArgsIter<'_> {
    pub fn index(&self) -> usize {
        self.index
    }

    fn peek(&mut self) -> Option<&(String, usize)> {
        self.iter.peek()
    }
}

impl Iterator for ArgsIter<'_> {
    type Item = (String, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| {
            self.index += 1;
            x
        })
    }
}


impl Args {
    pub fn new(a: &[&str]) -> Self {
        let a: Vec<&str> = a.into_iter().collect();
        let mut cmdline = String::new();

        let aa = a.into_iter().map(|arg| {
            let old_index = cmdline.len();

            if arg.contains(' ') {
                let new_arg = arg.replace('"', "\\\"").replace('\n', "\\n");
                cmdline.push('"');
                cmdline += &new_arg;
                cmdline.push('"');
            }
            else {
                let new_arg = arg.replace('\n', "\\n");
                cmdline += &new_arg;
            }
            cmdline.push(' ');

            (arg.to_owned(), old_index)
        }).into_vec();

        Args { args: aa, cmdline }
    }

    pub fn iter(&self) -> ArgsIter {
        ArgsIter{iter: self.args.iter().fuse().peekable(), index: 0}
    }

    pub fn cmdline(&self) -> &str {
        &self.cmdline
    }
}

impl Index<usize> for Args {
    type Output = (String, usize);

    fn index(&self, arg: usize) -> &Self::Output {
        &self.args[arg]
    }
}
