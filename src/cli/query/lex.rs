use fancy_regex::Regex;
use std::convert::TryFrom;

use crate::format::re::regex_expect;
use crate::format::str::{ToCharIterator, StrExtensions};
use crate::linq::collectors::IntoVec;
use super::lexeme::{LexemeQueue, LexemeKind, Lexeme};

use self::LexError::*;
use self::StringLiteralLexError::*;
use self::HexSequenceError::*;
use crate::cli::query::args::ArgsIter;
use crate::cli::query::lexeme::EqualityKind;

pub enum LexError {
    StringError(StringLiteralLexError),
    NonLexableSequence,
}

pub enum StringLiteralLexError {
    MissingClosingQuote,
    SyntaxError(usize)
}

pub enum HexSequenceError {
    NotEnoughChars,
    NonHexChar,
    NonUtf8Sequence
}

fn lex_string_literal(slice: &str) -> Result<usize, StringLiteralLexError> {
    use serde_json::Error::*;

    static QUOTE_REGEX: &Regex = regex_expect(r#"^("|').*?(?<!\)\1"#);

    let quot = match QUOTE_REGEX.find(slice)
        .expect("QUOTE_REGEX is invalid") {
        Some(s) => s,
        None => return Err(MissingClosingQuote)
    };

    if quot.start() != 0 {
        return Err(SyntaxError(0))
    }

    match serde_json::from_str(&slice[0..quot.end()]) {
        Ok(s) => Ok(quot.end()),
        Err(e) => Err(SyntaxError(e.column()))
    }
}

static ID_REGEX: &Regex = regex_expect(r"^[a-zA-Z0-9\-_]+\b");


static LITERAL_TOKENS: &[(&Regex, LexemeKind)] = &[
    (regex_expect(r"^,"), LexemeKind::Comma),
    (regex_expect(r"^\("), LexemeKind::LParen),
    (regex_expect(r"^\)"), LexemeKind::RParen),
    (regex_expect(r"^=="), LexemeKind::Equals(EqualityKind::Strict)),
    (regex_expect(r"^="), LexemeKind::Equals(EqualityKind::Strict)),
    (regex_expect(r"^is\b"), LexemeKind::Equals(EqualityKind::Strict)),
    (regex_expect(r"^in\b"), LexemeKind::In),
    (regex_expect(r"^and\b"), LexemeKind::And),
    (regex_expect(r"^or\b"), LexemeKind::Or),
    (regex_expect(r"^matches\b"), LexemeKind::Equals(EqualityKind::Matches))
];

fn get_token(slice: &str) -> Result<(usize, LexemeKind), LexError> {
    let slice = slice.trim_start();

    for (re, kind) in LITERAL_TOKENS {
        if let Some(m) = re.find(slice) {
            debug_assert_eq!(m.start(), 0);

            return Ok((m.end(), *kind));
        }
    }

    if slice.starts_with("'") || slice.starts_with("\"") {
        return match lex_string_literal(slice) {
            Ok(s) => Ok((s, LexemeKind::Value)),
            Err(e) => Err(LexError::StringError(e))
        };
    }

    if let Some(m) = ID_REGEX.find(slice) {
        debug_assert_eq!(m.start(), 0);

        return Ok ((m.end(), LexemeKind::Key));
    }

    Err(NonLexableSequence)
}

pub fn lex(args: ArgsIter, cmdline: &str) -> Result<LexemeQueue, LexError> {
    let a = args.flat_map(|(arg, index)| {
        if arg.contains(" ") {
            [("(", index), (arg, index), (")", index + arg.len() - 1)]
        }
        else {
            [(arg, index)]
        }
    }).collect::<Vec<(String, usize)>>();

    let mut ret = LexemeQueue::new();

    for (arg, index) in &a {
        let slice = arg.trim();

        while slice.len() > 0 {
            let tup = get_token(slice)?;
            ret.push(Lexeme::new(
                &slice[..tup.0],
                tup.1,
                (&cmdline[index + tup.0..]).slice_until(|c| c.is_whitespace()))
            );
        }
    }

    Ok(ret)
}