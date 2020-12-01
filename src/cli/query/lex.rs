use regex::Regex;
use std::convert::TryFrom;

use crate::format::re::regex_expect;
use crate::format::str::{ToCharIterator, StrExtensions};
use crate::linq::collectors::IntoVec;
use super::lexeme::{LexemeQueue, LexemeKind, Lexeme};

use self::LexError::*;
use self::StringLiteralLexError::*;
use self::HexSequenceError::*;

pub enum LexError {
    StringError(StringLiteralLexError),
    NonLexableSequence(String),
}

pub enum StringLiteralLexError {
    MissingClosingQuote,
    EndingEscape,
    UnknownEscape(String),
    HexSequenceError(HexSequenceError)
}

pub enum HexSequenceError {
    NotEnoughChars,
    NonHexChar,
    NonUtf8Sequence
}

/// Turns a sequence of HH into the character corresponding to that hexadecimal.
/// Returns Err if the given iterator does not produce two characters (`NotEnoughChars`), if the two characters do not form a hex literal (`NonHexChar`), or if the hexadecimal number cannot be converted to a UTF-8 character (`NonUtf8Sequence`).
///
/// Meant for use with lex_string_literal()
///
/// # Arguments
///
/// * `iter` - An iterator of characters. This must be fused, meaning once it produces None, it must continue producing None.
///
/// # Examples
/// ```
/// assert_eq!(x_sequence_to_char(['B', '8'].iter()), Ok('\xB8'));
/// assert_eq!(x_sequence_to_char(['b', '8'].iter()), Ok('\xB8'));
/// assert_eq!(x_sequence_to_char(['b', '8', 'q'].iter()), Ok('\xB8'));
/// assert_eq!(x_sequence_to_char(['b'].iter()), Err(NotEnoughChars));
/// assert_eq!(x_sequence_to_char(['b', 'r'].iter()), Err(NonHexChar));
/// ```
fn x_sequence_to_char<I: Iterator<Item=char>>(mut iter: I) -> Result<char, HexSequenceError> {
    match (iter.next(), iter.next()) {
        (_, None) => Err(NotEnoughChars),
        (Some(a), Some(b)) => {
            let s = [a, b].iter().collect::<String>();

            let n = match u8::from_str_radix(&s, 16) {
                Ok(n) => n,
                Err(_) => return Err(NonHexChar)
            };

            Ok(char::from(n))
        }
        _ => {panic!("Iterator passed to x_sequence_to_char() was not fuse()'d. This is a bug.")}
    }
}

/// Turns a sequence of HHHH into the character corresponding to that hexadecimal.
/// Returns Err if the given iterator does not produce two characters (`NotEnoughChars`), if the two characters do not form a hex literal (`NonHexChar`), or if the hexadecimal number cannot be converted to a UTF-8 character (`NonUtf8Sequence`)
///
/// Meant for use with lex_string_literal()
///
/// # Arguments
///
/// * `iter` - An iterator of characters. This must be fused, meaning once it produces None, it must continue producing None.
///
/// # Examples
/// ```
/// assert_eq!(u_sequence_to_char(['B', '8', '0', 'F'].iter()), Ok('\uB80F'));
/// assert_eq!(u_sequence_to_char(['b', '8', '0', 'f'].iter()), Ok('\xB80F'));
/// assert_eq!(u_sequence_to_char(['b', '8', '0', 'f', 'q'].iter()), Ok('\xB80F'));
/// assert_eq!(u_sequence_to_char(['b', '8', '0'].iter()), Err(NotEnoughChars));
/// assert_eq!(u_sequence_to_char(['b', '8', '0', 'r'].iter()), Err(NonHexChar));
/// ```
fn u_sequence_to_char<I: Iterator<Item=char>>(mut iter: &mut I) -> Result<char, HexSequenceError> {
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (_, _, _, None) => Err(NotEnoughChars),
        (Some(a), Some(b), Some(c), Some(d)) => {
            let s = [a, b, c, d].iter().collect::<String>();

            let n = match u32::from_str_radix(&s, 16) {
                Ok(n) => n,
                Err(_) => return Err(NonHexChar)
            };

            match char::try_from(n) {
                Ok(c) => Ok(c),
                Err(_) => Err(NotEnoughChars)
            }
        }
        _ => {panic!("Iterator passed to u_sequence_to_char() was not fuse()'d. This is a bug.")}
    }
}

fn lex_string_literal(slice: &str) -> Result<(String, &str), StringLiteralLexError> {
    debug_assert!(slice.starts_with("'") || slice.starts_with("\""));

    let mut iter = slice.char_iterator();

    let opening_quote = iter.next().unwrap();
    let mut ret = opening_quote.to_string();

    for c in iter {
        ret.push(if c == '\\' {
            match iter.next() {
                Some('n') => '\n',
                Some('r') => '\r',
                Some('\t') => '\t',
                Some('\\') => '\\',
                Some('\'') => '\'',
                Some('"') => '"',
                Some('x') => {
                    match x_sequence_to_char(&mut iter) {
                        Ok(c) => c,
                        Err(e) => return Err(StringLiteralLexError::HexSequenceError(e))
                    }
                },
                Some('u') => {
                    match u_sequence_to_char(&mut iter) {
                        Ok(c) => c,
                        Err(e) => return Err(StringLiteralLexError::HexSequenceError(e))
                    }
                },
                Some(c) => c,
                None => return Err(StringLiteralLexError::EndingEscape)
            }
        } else {
            if c == opening_quote {
                ret.push(c);
                return Ok((ret, &slice[..iter.position_bytes()]));
            }
            c
        });
    }

    Err(StringLiteralLexError::MissingClosingQuote)
}

static ID_REGEX: Regex = regex_expect(r"^[a-zA-Z0-9\-_]+\b");

static LITERAL_TOKENS: [(Regex, LexemeKind); 7] = [
    (regex_expect(r"^\("), LexemeKind::LParen),
    (regex_expect(r"^\)"), LexemeKind::RParen),
    (regex_expect(r"^=="), LexemeKind::Equals),
    (regex_expect(r"^="), LexemeKind::Equals),
    (regex_expect(r"^is\b"), LexemeKind::Equals),
    (regex_expect(r"^and\b"), LexemeKind::And),
    (regex_expect(r"^or\b"), LexemeKind::Or)
];

fn get_token(slice: &str) -> Result<(Lexeme, &str), LexError> {
    let slice = slice.trim_start();

    for (re, kind) in &LITERAL_TOKENS {
        if let Some(m) = re.find(slice) {
            debug_assert_eq!(m.start(), 0);

            return Ok(
                (Lexeme::new(m.as_str().to_owned(), *kind), &slice[m.end()..])
            );
        }
    }

    if slice.starts_with("'") || slice.starts_with("\"") {
        return match lex_string_literal(slice) {
            Ok(s) => Ok((Lexeme::new(s.0, LexemeKind::Value), s.1)),
            Err(e) => Err(LexError::StringError(e))
        };
    }

    if let Some(m) = ID_REGEX.find(slice) {
        debug_assert_eq!(m.start(), 0);

        return Ok (
            (Lexeme::new(m.as_str().to_owned(), LexemeKind::Identifier), &slice[m.end()..])
        );
    }

    Err(NonLexableSequence(slice.slice_until(char::is_whitespace).to_owned()))
}

pub fn lex(args: &[&str]) -> Result<LexemeQueue, LexError> {
    let s: String = args.iter().map(|arg| {
        if arg.contains(" ") {
            "(".to_owned() + arg + ")"
        }
        else {
            (*arg).to_owned()
        }
    }).into_vec().join(" ");

    let mut slice = s.trim();
    let mut ret = LexemeQueue::new();

    while slice.len() > 0 {
        let (lexeme, s) = match get_token(slice) {
            Ok(l) => l,
            Err(e) => return Err(e)
        };

        ret.push(lexeme);

        slice = s.trim_start();
    }

    Ok(ret)
}