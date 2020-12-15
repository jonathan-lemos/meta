use std::env;
use crate::linq::collectors::IntoVec;
use crate::format::re::regex_expect;
use fancy_regex::Regex;
use crate::cli::program::{program_name, authors, version, description};
use crate::format::str::StrExtensions;
use colored::{Colorize};
use crate::cli::query::args::{Args, ArgsIter};
use std::process::exit;
use crate::cli::query::parse::{OrQuery, ParseError, parse};
use std::collections::HashMap;
use crate::cli::args::SubcommandParseError::{MissingFlagValue, UnknownFlag, ExtraPositionalArgument, UnexpectedPositionalArgument, NotEnoughPositionalArguments};
use crate::cli::query::lex::{lex, LexError};

bitflags! {
    pub struct FileSelector: u8 {
        const NONE = 0x1;
        const FILE_LIST = 0x2;
        const QUERY = 0x4;
    }
}

pub enum FileEntryExpr {
    List(Vec<String>),
    Expr(OrQuery)
}

pub struct SubcommandParseResults {
    flags: Vec<(Flag, Option<String>, String)>,
    positional: Vec<String>,
    expr: Option<FileEntryExpr>,
}

pub enum SubcommandParseError<'a> {
    MissingFlagValue(&'a Flag),
    UnknownFlag(String, usize),
    LexError(LexError),
    ParseError(ParseError),
    UnexpectedPositionalArgument(String, usize),
    ExtraPositionalArgument(&'a Positional, String, usize),
    NotEnoughPositionalArguments(&'a Positional)
}

pub fn parse_subcommand<'a>(sc: &'a Subcommand, mut args: ArgsIter, cmdline: &str) -> Result<SubcommandParseResults, SubcommandParseError<'a>> {
    let mut flags = Vec::new();
    let mut positional = Vec::new();
    let mut expr = None;
    let long_map = sc.flags.iter()
        .map(|x| (x.long.trim_start_matches('-'), x))
        .collect::<HashMap<_, _>>();
    let short_map = sc.flags.iter()
        .filter(|x| x.short.is_some())
        .map(|x| (x.short.unwrap().trim_start_matches('-'), x))
        .collect::<HashMap<_, _>>();

    let mut parse_flags = true;

    'outer: for (arg, index) in args {
        if parse_flags {
            if arg == "--" {
                parse_flags = false;
                continue;
            }

            for (prefix, map) in &[("--", &long_map), ("-", &short_map)] {
                if arg.starts_with(prefix) {
                    let t = arg.trim_start_matches(prefix);

                    if let Some(s) = map.get(t) {
                        let eq = if s.equals_name.is_some() {
                            match args.next() {
                                Some(s) => Some(s.0),
                                None => return Err(MissingFlagValue(s))
                            }
                        } else {
                            None
                        };

                        flags.push(((*s).clone(), eq, arg));
                        continue 'outer;
                    }

                    if t.contains("=") {
                        let spl = t.splitn(2, '=').into_vec();
                        let (key, value) = (spl[0], spl.get(1));

                        if let Some(s) = map.get(key) {
                            let eq = if s.equals_name.is_some() {
                                match value {
                                    Some(s) => Some(s),
                                    None => return Err(MissingFlagValue(s))
                                }
                            } else {
                                None
                            };

                            flags.push(((*s).clone(), eq.map(|x| x.to_string()), arg));
                            continue 'outer;
                        }
                    }
                }
            }

            if arg.starts_with("-") {
                return Err(UnknownFlag(arg, index));
            }
        }

        if arg == "where" {
            let mut lexemes = lex(args, cmdline).map_err(SubcommandParseError::LexError)?;
            let query = parse(&mut lexemes).map_err(SubcommandParseError::LexError)?;
            expr = Some(FileEntryExpr::Expr(query));
            break;
        }

        if arg == "from" {
            let vec = args.map(|x| x.0).into_vec();
            expr = Some(FileEntryExpr::List(vec));
            break;
        }

        match &sc.positional {
            Some(p) => {
                positional.push(arg);
                if let Some(max) = p.count.1 {
                    if positional.len() > max {
                        return Err(ExtraPositionalArgument(p, arg.clone(), index))
                    }
                }
            }
            None => return Err(UnexpectedPositionalArgument(arg, index))
        }

    }

    if let Some(p) = &sc.positional {
        if positional.len() < p.count.1.unwrap_or(0) {
            return Err(NotEnoughPositionalArguments(p))
        }
    }

    Ok(SubcommandParseResults {flags, positional, expr})
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Positional {
    pub(crate) name: &'static str,
    pub(crate) count: (Option<usize>, Option<usize>),
    pub(crate) description: &'static str,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Flag {
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) equals_name: Option<&'static str>,
    pub(crate) description: &'static str,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Subcommand {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) positional: Option<Positional>,
    pub(crate) file_selector: FileSelector,
    pub(crate) flags: Vec<Flag>,
    pub(crate) on_parse: dyn FnOnce(SubcommandParseResults),
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

pub static HELP_FLAG: Flag = Flag {
    aliases: vec!["--help", "-h"],
    equals_name: None,
    description: "Displays the help for this command."
};

pub static QUIET_FLAG: Flag = Flag {
    aliases: vec!["--quiet", "q"],
    equals_name: None,
    description: "Displays less information about what the command is doing."
};

pub static RECURSIVE_FLAG: Flag = Flag {
    aliases: vec!["--recursive", "-r"],
    equals_name: None,
    description: "The command will be recursively applied to the contents of any directories given."
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
        file_entry_expr: true,
        flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG],
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
        file_entry_expr: true,
        flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG],
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
        file_entry_expr: true,
        flags: vec![HELP_FLAG, QUIET_FLAG, Flag {
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


static WIDTH: Option<usize> = term_size::dimensions_stdout().map(|x| x.1);

fn println_col(begin: usize, s: String) {
    println!("{}", &s.col_begin_end(begin, WIDTH));
}

fn print_col(begin: usize, s: String) {
    print!("{}", &s.col_begin_end(begin, WIDTH));
}

pub fn print_version() {
    println_col(0, format!("{} {}", program_name(), version().bold().yellow()));
}

pub fn print_help(prog_invocation: &str) {
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

    // meta 1.0.0
    pln(0, format!("{} {}", program_name().bold().yellow(), version()));
    // A command-line utility for managing file/directory metadata.
    pln(0, description().to_owned());

    // USAGE:
    pln(0, "USAGE:".to_owned());
    //     meta [flags*] [subcommand] [subcommand-argument*]
    pln(INDENT, format!("{} {} {} {}", prog_invocation, "[flags*]".italic(), "[subcommand]".italic(), "[subcommand-argument*]".italic()));

    println!();
    // FLAGS:
    pln(0, "FLAGS:".to_owned());

    for (flag, desc) in global_flags.iter().zip(global_flag_desc.iter()) {
        let desc_begin = INDENT + global_flag_desc_len.unwrap() + INDENT;
        // -f, --flag
        print!("{}", flag.col_begin_end_indent(INDENT, width));
        //               description
        pln(desc_begin, desc.clone());
        println!();
    }

    println!();
    // SUBCOMMANDS:
    pln(0, "SUBCOMMANDS:".to_owned());

    for subcommand in comm {
        let desc_begin = INDENT + global_subcomm_len.unwrap() + INDENT;
        // subcommand
        print!("{}", subcommand.name.col_begin_end_indent(INDENT, width));
        //               description
        pln(desc_begin, subcommand.description.to_owned());
        println!();
    }

    pln(0, format!("For more details about a subcommand, type {} {} {}", prog_invocation, "[subcommand]", "--help"))
}

pub fn parse_command_line_args() {
    let raw = env::args().into_vec();
    let args = Args::new(raw.iter().collect());

    for (arg, index) in args.iter().skip(1) {
        match arg.to_lowercase().as_str() {
            "--help" | "-h" | "help" => {
                print_help(&args[0].0);
                exit(0);
            },
            "--version" | "-v" | "version" => {
                print_version();
                exit(0);
            }
            _ => {}
        }


    }
}

