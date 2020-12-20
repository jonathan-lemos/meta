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
use super::subcommands::{get, list, remove, set};
use crate::cli::help::{print_help, print_version};
use crate::cli::print::{log, Logger};
use crate::cli::typo::typos_threshold;
use crate::cli::lang;

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

pub struct ArgError {
    pub arg: String,
    pub position: usize,
    pub cmdline: String
}

impl ArgError {
    pub fn new(arg: String, position: usize, cmdline: &str) -> Self {
        ArgError { arg, position, cmdline: cmdline.to_owned() }
    }
}

pub enum SubcommandParseError<'a> {
    MissingFlagValue(&'a Flag, ArgError),
    UnknownFlag(ArgError),
    LexError(LexError),
    ParseError(ParseError),
    UnexpectedPositionalArgument(ArgError),
    ExtraPositionalArgument(&'a Positional, ArgError),
    NotEnoughPositionalArguments(&'a Positional)
}

pub fn parse_subcommand<'a, I: Iterator<Item=(String, usize)>>(sc: &'a Subcommand, mut args: I, cmdline: &str) -> Result<SubcommandParseResults, SubcommandParseError<'a>> {
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
                                None => return Err(MissingFlagValue(s, ArgError::new(arg, index, cmdline)))
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
                                    None => return Err(MissingFlagValue(s, ArgError::new(arg, index, cmdline)))
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
                return Err(UnknownFlag(ArgError::new(arg, index, cmdline)));
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
                        return Err(ExtraPositionalArgument(p, ArgError::new(arg, index, cmdline)))
                    }
                }
            }
            None => return Err(UnexpectedPositionalArgument(ArgError::new(arg, index, cmdline)))
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
    pub(crate) on_parse: Box<dyn FnOnce(SubcommandParseResults)>,
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
    get::SUBCOMMAND,
    list::SUBCOMMAND,
    set::SUBCOMMAND,
    remove::SUBCOMMAND
];

static FLAGS: &[Flag] = &[
    HELP_FLAG
];

pub fn parse_command_line_args() -> () {
    let raw = env::args().into_vec();
    let args = Args::new(raw.iter().collect());
    let a = args.iter().skip(1);

    for (arg, index) in a {
        match arg.to_lowercase().as_str() {
            "--help" | "-h" | "help" => {
                print_help(SUBCOMMANDS, FLAGS, &args[0].0);
                exit(0);
            },
            "--version" | "-v" | "version" => {
                print_version();
                exit(0);
            }
            _ => {}
        }

        for sc in SUBCOMMANDS {
            if sc.name == arg.to_lowercase() {
                let log_cmdline = || log().cmdline(&a.cmdline, a.position, a.arg.chars().count());

                let res = match parse_subcommand(sc, a, args.cmdline()) {
                    Ok(s) => s,
                    Err(e) => match e {
                        MissingFlagValue(e, a) => {
                            log().error(&format!("The flag {0} is missing a value. Specify {0}={1} or {0} {1}", a.arg.bold().yellow(), "value".green().italic()));
                            log_cmdline();
                            exit(0);
                        }
                        UnknownFlag(a) => {
                            if sc.flags.len() == 0 {
                                log().error(&format!(
                                    "A flag {0} was given, but the {1} subcommand does not accept any flags. Type {2} {1} --help for more information. If you want to specify {0} as a positional argument, put it after a {3} argument.",
                                    a.arg.bold().red(),
                                    sc.name.bold().yellow(),
                                    args[0].0,
                                    "--".bold().green()
                                ));
                                log_cmdline();
                                return;
                            }

                            let typos = typos_threshold(&a.arg, sc.flags.iter().flat_map(|x| x.aliases), 0.25, 2);
                            let or = lang::or(typos.iter().map(|x| x.0)).split(", ").map(|x| x.yellow().bold()).into_vec().join(", ");

                            if typos.len() == 0 {
                                log().error(&format!{
                                    "An unknown flag {0} was given. Type {2} {1} --help for a list of accepted flags. If you want to specify {0} as a positional argument, put it after a {3} argument.",
                                    a.arg.bold().red(),
                                    sc.name.bold().yellow(),
                                    args[0].0,
                                    "--".bold().green()
                                });
                            }
                            else {
                                log().error(&format!{
                                    "An unknown flag {0} was given. Did you mean {1}? Type {3} {2} --help for a list of accepted flags. If you want to specify {0} as a positional argument, put it after a {4} argument.",
                                    a.arg.bold().red(),
                                    or,
                                    sc.name.bold().yellow(),
                                    args[0].0,
                                    "--".bold().green()
                                });
                            }
                            log_cmdline();
                        }
                        SubcommandParseError::LexError(a) => {
                            log().error(&format!("The token {0} was unrecognized.", a.arg.bold().red()));
                            log_cmdline();
                        }
                        SubcommandParseError::ParseError(a) => {
                        }
                        UnexpectedPositionalArgument(_, _) => {}
                        ExtraPositionalArgument(_, _, _) => {}
                        NotEnoughPositionalArguments(_) => {}
                    }
                }
                return;
            }
        }
    }
}

