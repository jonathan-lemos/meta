use std::env;
use crate::linq::collectors::IntoVec;
use std::ops::Range;
use PickerResult::*;
use Count::*;
use crate::format::re::regex_expect;
use regex::Regex;

pub struct Options {
    subcommand: String
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Count {
    Range(Range<usize>),
    Func(dyn FnMut(&[&str]) -> PickerResult)
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum PickerResult {
    More,
    PossibleMore,
    Complete,
    Invalid(String),
    Less
}

struct Positional {
    name: &'static str,
    count: Count,
    description: &'static str,
}

struct Flag {
    long: &'static str,
    short: Option<&'static str>,
    value_name: Option<&'static str>,
    description: &'static str,
}

struct Subcommand {
    name: &'static str,
    description: &'static str,
    positional: Vec<Positional>,
    flags: Vec<Flag>,
    subcommands: Vec<Subcommand>,
}

pub static ASSIGN_RE: Regex = regex_expect(r"^([a-zA-Z0-9_-]+)=(.+)$");

pub static KEY_RE: Regex = regex_expect("^([a-zA-Z0-9_-])+$");

/// Turns a list of comma separated arguments into their corresponding arguments without commas.
///
/// If a double comma is present, the index of the argument with the second comma is returned.
/// Otherwise, a vector of string slices from the original argument array is returned.
///
/// Examples:
/// ```
/// assert_eq!(key_list(&["abc", "def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc,def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc,", "def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc", ",def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc, def"]), Ok(vec!["abc", "def"]));
/// ```
fn key_list(args: &[&str]) -> Result<Vec<&str>, usize> {

    let mut ret = Vec::<&str>::new();
    let mut comma: bool = false;

    let split_re = regex_expect(r"\s*,\s*");

    for (i, arg) in args.into_iter().enumerate() {
        if comma && arg.starts_with(",") {
            return Err(i);
        }

        if arg.contains(",,") {
            return Err(i);
        }

        for a in arg.trim_matches(",").split(&split_re) {
            ret.push(a);
        }

        comma = arg.ends_with(",");
    }

    Ok(ret)
}

static SUBCOMMANDS: &[Subcommand] = &[
    Subcommand {
        name: "get",
        description: "Retrieves key/value pairs.",
        positional: vec![
            Positional {
                name: "([key,]*key)?",
                count: Func(|strs| {
                    if strs.len() == 0 {
                        return PossibleMore
                    }

                    strs = strs.into_vec().join(",");



                }),
                description: "The command will print the values for the given keys. If no keys are given, it will print all key/value pairs."
            }
        ],
        flags: vec![],
        subcommands: vec![]
    },
    Subcommand {
        name: "set",
        description: "Sets the value associated with a key.",
        positional: vec![
            Positional {
                name: "(key=value)+",
                count: Func(|strs| {
                    match strs.len() {
                        0 => More,
                        _ => {
                            if ASSIGN_RE.is_match(strs.last().unwrap()) {
                                PossibleMore
                            }
                            else {
                                if strs.len() == 1 {Invalid} else {Less}
                            }
                        }
                    }
                }),
                description: "One or more key=value assignments, meaning assign the value to the key."
            }
        ],
        flags: vec![],
        subcommands: vec![]
    },
    Subcommand {
        name: "remove",
        description: "Removes the value associated with a key."
    },
]

pub fn parse_command_line_args() {
    let raw = env::args().into_vec();


}

