use crate::linq::collectors::IntoVec;
use std::ops::Index;
use std::slice::Iter;

pub struct Args {
    args: Vec<(String, usize)>,
    cmdline: String
}

pub type ArgsIter<'a> = Iter<'a, (String, usize)>;

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
        self.args.iter()
    }

    pub fn cmdline(&self) -> &str {
        &self.cmdline
    }
}

impl Index<usize> for Args {
    type Output = String;

    fn index(&self, arg: usize) -> &Self::Output {
        &self.args[arg].0
    }
}
