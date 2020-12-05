use std::env;
use crate::linq::collectors::IntoVec;
use std::ops::Range;
use PickerResult::*;
use Count::*;
use crate::format::re::regex_expect;
use regex::Regex;
use crate::cli::program::{program_name, authors, version, description};
use crate::format::str::StrExtensions;

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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Positional {
    name: &'static str,
    count: Count,
    description: &'static str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Flag {
    long: &'static str,
    short: Option<&'static str>,
    equals_name: Option<&'static str>,
    description: &'static str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Subcommand {
    name: &'static str,
    description: &'static str,
    positional: Vec<Positional>,
    positional_expr: bool,
    flags: Vec<Flag>,
}

pub static ASSIGN_RE: &Regex = regex_expect(r"^([a-zA-Z0-9_-]+)=(.+)$");

pub static KEY_RE: &Regex = regex_expect("^([a-zA-Z0-9_-])+$");

/// Turns a list of comma separated arguments into their corresponding arguments without commas.
///
/// If a double comma is present, the index of the argument with the second comma is returned.
/// Otherwise, a vector of string slices from the original argument array is returned.
///
/// A trailing comma is allowed.
///
/// Examples:
/// ```
/// assert_eq!(key_list(&["abc", "def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc,def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc,", "def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc", ",def"]), Ok(vec!["abc", "def"]));
/// assert_eq!(key_list(&["abc, def"]), Ok(vec!["abc", "def"]));
/// ```
fn key_list<'a>(args: &[&'a str]) -> Result<Vec<&'a str>, usize> {
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

static KEYWORDS: &[&str] = &["from", "where"];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.iter().any(|kw| s == kw)
}

pub fn key_count(strs: &[&str]) -> PickerResult {
    if strs.len() == 0 {
        return PossibleMore
    }

    match key_list(strs) {
        Ok(s) => {
            if s.last().filter(|s| is_keyword(&s.to_lowercase())).is_some() {
                Less
            }
            else {
                PossibleMore
            }
        },
        Err(e) => Invalid(format!("Double comma detected in {}", strs.last().unwrap()))
    }
}

static HELP_FLAG: Flag = Flag {
    long: "--help",
    short: Some("-h"),
    equals_name: None,
    description: "Displays the help for this command."
};

static VERBOSE_FLAG: Flag = Flag {
    long: "--quiet",
    short: Some("-q"),
    equals_name: None,
    description: "Displays less information about what the command is doing."
};

static SUBCOMMANDS: &[Subcommand] = &[
    Subcommand {
        name: "get",
        description: "Retrieves key/value pairs.",
        positional: vec![
            Positional {
                name: "([key,]*key)?",
                count: Func(key_count),
                description: "The command will print the values for the given keys. If no keys are given, it will print all key/value pairs."
            }
        ],
        positional_expr: true,
        flags: vec![HELP_FLAG, VERBOSE_FLAG],
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
        positional_expr: true,
        flags: vec![HELP_FLAG, VERBOSE_FLAG],
    },
    Subcommand {
        name: "remove",
        description: "Removes metadata.",
        positional: vec![
            Positional {
                name: "([key,]*key)*",
                count: Func(key_count),
                description: "The command will remove the given keys."
            }
        ],
        positional_expr: true,
        flags: vec![HELP_FLAG, VERBOSE_FLAG, Flag {
            long: "--all",
            short: Some("-a"),
            equals_name: None,
            description: "Removes all of the keys from the given targets."
        }],
    },
];

static FLAGS: &[Flag] = &[
    HELP_FLAG
];

pub fn print_help() {
    const INDENT: usize = 4;
    let width = term_size::dimensions_stdout().map(|x| x.1);

    let pln = |begin: usize, s: String| {
        println!("{}", &s.col_begin_end(begin, width))
    };

    let prt = |begin: usize, s: String| {
        print!("{}", &s.col_begin_end(begin, width))
    };

    let flag_desc = |f: &Flag| {
        match f.short {
            Some(s) => f.long.to_owned() + ", " + s,
            None => f.long.to_owned()
        }
    };

    let mut comm = SUBCOMMANDS.clone();
    comm.sort_by_key(|k| k.name);
    let global_subcomm_len = comm.iter().map(|x| x.name.len()).max();

    let mut global_flags = FLAGS.clone();
    global_flags.sort_by_key(|k| k.long);
    let global_flag_desc = global_flags.iter().map(flag_desc).into_vec();
    let global_flag_desc_len = global_flag_desc.iter().map(|x| x.len()).max();

    pln(0, format!("{} {}", program_name(), version()));
    pln(0, format!("{}", authors().into_vec().join("\n")));
    pln(0, description().to_owned());

    println!();
    pln(0, "FLAGS:".to_owned());

    for (flag, desc) in global_flags.iter().zip(global_flag_desc.iter()) {
        let desc_begin = INDENT + global_flag_desc_len.unwrap() + INDENT;
        print!("{}", flag.col_begin_end_indent(INDENT, width));
        pln(desc_begin, desc.clone());
        println!();
    }

    println!();
    pln(0, "SUBCOMMANDS:".to_owned());

    for subcommand in comm {
        let desc_begin = INDENT + global_subcomm_len.unwrap() + INDENT;
        print!("{}", subcommand.name.col_begin_end_indent(INDENT, width));
        pln(desc_begin, subcommand.description.to_owned());
        println!();
    }
}

pub fn parse_command_line_args() {
    let raw = env::args().into_vec();


}

