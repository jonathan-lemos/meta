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
use crate::cli::query::parse::HexSequenceError::{NotEnoughChars, NonHexChar, NonUtf8Sequence};
use regex::Regex;
use crate::format::str::{ToCharIterator, StrExtensions};
use crate::cli::query::parse::LexError::NonLexableSequence;
use std::mem::take;
use crate::cli::query::parse::Factor::KeyEqualsValue;
use crate::cli::query::lex::LexError;
use crate::cli::query::lexeme::LexemeQueue;

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
    KeyEqualsValue((String, String))
}

pub enum ParseError {
    UnexpectedEOF(String),
    InvalidSequence(String),
    LexError(LexError)
}

pub fn parse(args: &[&str]) -> Result<OrQuery, ParseError> {
    let lexemes = lex(args);

    None
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
        else {
            None
        }
    })
}

pub fn parse_factor(lexemes: &mut &[Lexeme]) -> Result<Factor, ParseError> {
    let tok = match pop(lexemes) {
        Some(t) => t,
        None => ParseError::UnexpectedEnding(vec!(LexemeKind::LParen, LexemeKind::Equals))
    };

    match tok.kind {
        LexemeKind::LParen => {
            let expr = parse_or_query(lexemes)?;
            pop_kind(lexemes, &vec![LexemeKind::RParen])?;
            return Ok(Factor::Query(Box::new(expr)));
        },
        LexemeKind::Identifier => {
            let next = match pop(lexemes) {
                Some(s) => s,
                None => return Err(ParseError::UnexpectedEnding(vec![LexemeKind::Equals, LexemeKind::Value, LexemeKind::Identifier]))
            };

            match next.kind {
                LexemeKind::Value | LexemeKind::Identifier => {
                    Ok(Factor::KeyEqualsValue((tok.str, next.str)))
                }
                LexemeKind::Equals => {
                    let val = pop_kind(lexemes, LexemeKind::Value | LexemeKind::Identifier)?;
                    Ok(Factor(KeyEqualsValue((tok.str, next.str))))
                }
            }
        }
    }
}


fn parse_expr(args: &mut Vec<String>) {

}