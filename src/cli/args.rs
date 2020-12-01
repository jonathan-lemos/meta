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

static SUBCOMMANDS: &[Subcommand] = &[
    Subcommand {
        name: "get",
        description: "Retrieves key/value pairs.",
        positional: vec![
            Positional {
                name: "(key(=value)?)?",
                count: Range(0..1),
                description: "If a key is given, this command will print the values for each file with that key. If a key and value are given, this command will print the entries that match both the key and the value."
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
                        1 => {
                            if ASSIGN_RE.is_match(strs.last().unwrap()) {
                                PossibleMore
                            }
                            else {
                                Invalid
                            }
                        }
                    }
                })
            }
        ]
    },
    Subcommand {
        name: "remove",
        description: "Removes the value associated with a key."
    },
]

pub fn parse_command_line_args() {
    let raw = env::args().into_vec();


}

