use crate::linq::collectors::IntoVec;
use std::cmp::min;
use std::convert::TryFrom;
use std::iter::Fuse;
use crate::cli::argparse::HexSequenceError::{NotEnoughChars, NonHexChar, NonUtf8Char, NonUtf8Sequence};

pub enum Operator {
    And,
    Or
}

pub struct ExprNode {
    left: Box<Node>,
    op: Operator,
    right: Box<Node>,
}

pub enum Terminal {
    Key(String),
    KeyEqualsValue((String, String))
}

pub enum Node {
    Expr(ExprNode),
    Terminal(Terminal)
}

pub enum ParseError {
    NoArguments,
    UnexpectedToken((usize, String)),
    EndOfFile(String)
}

pub enum TokenLexError {
    StringError(StringLiteralLexError),
    UnexpectedSymbol((String, Vec<String>)),
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

fn lex_string_literal(slice: &str) -> Result<String, StringLiteralLexError> {
    debug_assert!(slice.starts_with("'") || slice.starts_with("\""));

    let mut iter = slice.chars().fuse();

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
                return Ok(ret)
            }
            c
        });
    }

    Err(StringLiteralLexError::MissingClosingQuote)
}

fn get_token(slice: &str) -> Result<(&str, &str), LexTokenError> {
    let slice = slice.trim();

    let basic_tokens = ["(", ")", "=", "and", "or"];

    for bt in &basic_tokens {
        if slice.starts_with(bt) {
            return Ok((&slice[..bt.len()], &slice[bt.len()..]));
        }
    }

    if slice.starts_with("'") || slice.starts_with("\"") {
        let st = lex_string_literal(slic)
    }

    (slice, slice)
}

pub fn lex(args: &[&str]) -> Result<Vec<String>, String> {
    let s: String = args.iter().map(|arg| {
        if arg.contains(" ") {
            "(".to_owned() + arg + ")"
        }
        else {
            (*arg).to_owned()
        }
    }).into_vec().join(" ");

    let mut slice = s.trim();
    let mut ret = Vec::<String>::new();

    while slice.len() > 0 {
        let mut count = 0;


        ret.push(lexeme.to_owned());

        slice = slice.trim();
    }

    Ok(ret)
}

pub fn parse(args: &[&str]) -> Option<Node> {
    fn split_keep_special(text: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut last = 0;
        let chars = ['(', ')', '='];

        for (index, matched) in text.match_indices(&chars[..]) {
            if last != index {
                result.push(&text[last..index]);
            }
            result.push(matched);
            last = index + matched.len();
        }

        if last < text.len() {
            result.push(&text[last..]);
        }
        result
    }

    let mut lexemes = Vec::<&str>::new();

    if args.len() == 0 {
        return None;
    }

    for arg in args {
        let a = arg.trim();

        if a.contains(" ") {
            let pieces = a.split_whitespace().into_vec();

            if pieces.first().unwrap().chars().next() != Some('(') {
                lexemes.push("(");
            }

            for piece in &pieces {
                let sans = split_keep_special(piece);

                for p in sans {
                    lexemes.push(p);
                }
            }

            if pieces.last().unwrap().chars().last() != Some(')') {
                lexemes.push(")");
            }
        }
        else {
            lexemes.push(a);
        }
    }

    None
}

fn parse_expr(args: &mut Vec<String>) {

}