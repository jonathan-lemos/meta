/// Parses a query from command-line arguments.
/// The grammar for a query is as follows:
///
/// or-query -> and-query or or-query | and-query
/// and-query -> factor and and-query | factor
/// factor -> ( query ) | key | key is value // (command-line arguments in quotes e.g. 'this and that' are treated as being in parentheses)
/// key -> [a-zA-Z0-9\-_]+
/// eq -> == | =
/// value -> quotation | string
/// quotation -> "quote" | 'quote' (usual quote syntax)
/// string -> .* (single line)

use crate::linq::collectors::IntoVec;
use std::convert::TryFrom;
use crate::cli::queryparse::HexSequenceError::{NotEnoughChars, NonHexChar, NonUtf8Sequence};
use regex::Regex;
use crate::format::str::{ToCharIterator, StrExtensions};
use crate::cli::queryparse::TokenLexError::NonLexableSequence;

#[derive(Debug, Eq, PartialEq, Clone)]
struct Lexeme {
    str: String,
    kind: LexemeKind
}

impl Lexeme {
    pub fn new(str: String, kind: LexemeKind) -> Self {
        Lexeme {str, kind}
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum LexemeKind {
    LParen,
    RParen,
    Equals,
    Or,
    And,
    Identifier,
    Value
}

pub struct OrQuery {
    and_query: AndQuery,
    next: Option<Box<OrQuery>>
}

pub struct AndQuery {
    factor: Factor,
    next: Option<Box<AndQuery>>
}

pub enum Factor {
    Query(Box<OrQuery>),
    Key(String),
    KeyEqualsValue((String, Value))
}

pub enum Value {
    String(String),
    Quotation(String)
}

pub enum ParseError {
    UnexpectedEnding,
    LexError(TokenLexError)
}

pub enum TokenLexError {
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

fn regex_expect(regex: &str) -> Regex {
    match Regex::new(regex) {
        Ok(r) => r,
        Err(e) => panic!(format!("Invalid regex '{}': {}", regex, e))
    }
}

static ID_REGEX: Regex = regex_expect(r"^[a-zA-Z0-9\-_]+\b");

static LITERAL_TOKENS: [(Regex, LexemeKind); 7] = [
    (regex_expect(r"^\("), LexemeKind::LParen),
    (regex_expect(r"^\)"), LexemeKind::RParen),
    (regex_expect(r"=="), LexemeKind::Equals),
    (regex_expect(r"^="), LexemeKind::Equals),
    (regex_expect(r"^is\b"), LexemeKind::Equals),
    (regex_expect(r"^and\b"), LexemeKind::And),
    (regex_expect(r"^or\b"), LexemeKind::Or)
];

fn get_token(slice: &str) -> Result<(Lexeme, &str), TokenLexError> {
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
            Err(e) => Err(TokenLexError::StringError(e))
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

pub fn lex(args: &[&str]) -> Result<Vec<Lexeme>, TokenLexError> {
    let s: String = args.iter().map(|arg| {
        if arg.contains(" ") {
            "(".to_owned() + arg + ")"
        }
        else {
            (*arg).to_owned()
        }
    }).into_vec().join(" ");

    let mut slice = s.trim();
    let mut ret = Vec::<Lexeme>::new();

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

pub fn parse(args: &[&str]) -> Result<OrQuery, ParseError> {
    let lexemes = lex(args);

    None
}

fn peek(lexemes: &[Lexeme]) -> Option<&Lexeme> {
    lexemes.get(0)
}

fn peek_kind(lexemes: &[Lexeme]) -> Option<LexemeKind> {
    peek(lexemes).map(|x| x.kind)
}

fn pop(lexemes: &mut &[Lexeme]) -> Option<&Lexeme> {
    let ret = match peek(lexemes) {
        Some(s) => s,
        None => return None
    };

    *lexemes = &lexemes[1..];
    Some(ret)
}

pub fn parse_or_query(lexemes: &mut &[Lexeme]) -> Result<OrQuery, ParseError> {
    let and_query = parse_and_query(lexemes)?;

    Ok(OrQuery {
        and_query,
        next:
        if peek_kind(lexemes) == LexemeKind::Or {
            pop(lexemes);
            Some(Box::new(parse_or_query(lexemes)?))
        }
        else {
            None
        }
    })
}

pub fn parse_and_query(lexemes: &mut &[Lexeme]) -> Result<AndQuery, ParseError> {
    let factor = parse_factor(lexemes)?

    Ok(AndQuery {
        factor,
        next:
        if peek_kind(lexemes) == LexemeKind::And {
            pop(lexemes);
            Some(Box::new(parse_and_query(lexemes)?))
        }
    })
}

pub fn parse_factor(lexemes: &mut &[Lexeme]) -> Result<Factor, ParseError> {

}


fn parse_expr(args: &mut Vec<String>) {

}