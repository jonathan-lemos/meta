/// Parses a query from command-line arguments.
/// The grammar for a query is as follows:
///
/// or-query -> and-query or or-query | and-query
/// and-query -> factor and and-query | factor
/// factor -> ( or-query ) | key | key equals value | key in ( values ) // (command-line arguments in quotes e.g. 'this and that' are treated as being in parentheses)
/// key -> [a-zA-Z0-9\-_]+
/// equals -> = | == | is | matches
/// values -> value values | value , values | value
/// value -> key | quotation
/// quotation -> [[json quote]]

use crate::linq::collectors::IntoVec;
use std::convert::TryFrom;
use crate::cli::query::parse::HexSequenceError::{NotEnoughChars, NonHexChar, NonUtf8Sequence};
use regex::Regex;
use crate::format::str::{ToCharIterator, StrExtensions};
use crate::cli::query::parse::LexError::NonLexableSequence;
use std::mem::take;
use crate::cli::query::parse::Factor::KeyEqualsValue;
use crate::cli::query::lex::{LexError, lex};
use crate::cli::query::lexeme::{LexemeQueue, LexemeKind, Lexeme, EqualityKind};

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
    KeyEqualsValue((String, EqualityKind, String)),
    KeyIn((String, Vec<String>))
}

pub enum ParseError<'a, 'b> {
    UnexpectedEOF(String),
    UnexpectedToken((Lexeme<'a, 'b>, String)),
    TrailingToken(Lexeme<'a, 'b>)
}

pub fn parse(lexemes: &mut LexemeQueue) -> Result<OrQuery, ParseError> {
    let or_query = parse_or_query(lexemes)?;

    if let Some(s) = lexemes.peek() {
        return Err(ParseError::TrailingToken(s.clone()))
    }

    Ok(or_query)
}

pub fn parse_or_query(lexemes: &mut LexemeQueue) -> Result<OrQuery, ParseError> {
    let and_query = parse_and_query(lexemes)?;

    Ok(OrQuery {
        and_query,
        next:
        if let Some(s) = lexemes.pop_kind(LexemeKind::Or) {
            Some(Box::new(parse_or_query(lexemes)?))
        }
        else {
            None
        }
    })
}

pub fn parse_and_query(lexemes: &mut LexemeQueue) -> Result<AndQuery, ParseError> {
    let factor = parse_factor(lexemes)?;

    Ok(AndQuery {
        factor,
        next:
        if let Some(s) = lexemes.pop_kind(LexemeKind::And) {
            Some(Box::new(parse_and_query(lexemes)?))
        }
        else {
            None
        }
    })
}

pub fn parse_factor(lexemes: &mut LexemeQueue) -> Result<Factor, ParseError> {
    let tok = match lexemes.pop() {
        Some(t) => t,
        None => return Err(ParseError::UnexpectedEOF("Expected a key or a subquery, but no more arguments were available. Most likely you forgot to fill in the right half of an 'and' or 'or' query.".to_owned()))
    };

    match tok.kind() {
        LexemeKind::LParen => {
            let expr = parse_or_query(lexemes)?;

            let rparen = lexemes.pop();
            match rparen {
                Some(s) => {
                    if s.kind() != LexemeKind::RParen {
                        return Err(ParseError::UnexpectedEOF(("Expected a ')'. Most likely you forgot to include a closing ')'".to_owned())))
                    }
                    Ok(Factor::Query(Box::new(expr)))
                }
                None => Err(ParseError::UnexpectedToken((tok, "Expected a ')'. Most likely you forgot to include a closing ')'".to_owned())))
            }
        },
        LexemeKind::Key => {
            let next = match lexemes.pop() {
                Some(s) => s,
                None => return Ok(Factor::Key(tok.token().to_owned()))
            };

            match next.kind() {
                LexemeKind::In => {
                    let lparen = lexemes.pop();
                    match lparen {
                        Some(s) => {
                            if s.kind() != LexemeKind::LParen {
                                return Err(ParseError::UnexpectedToken((s, "Expected '(' after 'in'".to_owned())))
                            }
                        }
                        None => return Err(ParseError::UnexpectedEOF("Expected ')' after 'in'. Most likely you have an extra trailing 'in'.".to_owned()))
                    }

                    let values = parse_values(lexemes)?;

                    lexemes.pop_kind(LexemeKind::Comma);

                    let rparen = lexemes.pop();
                    match rparen {
                        Some(s) => {
                            if s.kind() != LexemeKind::RParen {
                                return Err(ParseError::UnexpectedToken((s, "Expected ')' to close the 'in' values. Most likely you forgot to include the closing ')'.".to_owned())))
                            }
                        }
                        None => return Err(ParseError::UnexpectedEOF("Expected ')' to close the 'in' values. Most likely you forgot to include the closing ')'.".to_owned()))
                    }

                    Ok(Factor::KeyIn((tok.token().to_owned(), values)))
                }
                LexemeKind::Equals(e) => {
                    let val = lexemes.pop();
                    match val {
                        Some(s) => {
                            if s.kind() != LexemeKind::Key && s.kind() != LexemeKind::Value {
                                return Err(ParseError::UnexpectedToken((s, "Expected a key or a value.".to_owned())))
                            }
                        }
                        None => return Err(ParseError::UnexpectedEOF(format!("Expected a value after '{}'.", next.token())))
                    }
                    Ok(Factor(KeyEqualsValue((tok.token().to_owned(), e, val.token().to_owned()))))
                },
                _ => Err(ParseError::UnexpectedToken((next, "Expected 'in', '=', '==', or 'matches'.".to_owned())))
            }
        }
        _ => Err(ParseError::UnexpectedToken((tok, "Expected '(' or a key.".to_owned())))
    }
}

pub fn parse_values(lexemes: &mut LexemeQueue) -> Result<Vec<String>, ParseError> {
    let mut ret = Vec::new();

    while let Some(s) = lexemes.pop_predicate(|l| l.kind() == LexemeKind::Key || l.kind() == LexemeKind::Value) {
        ret.add(s.token().to_owned());
        lexemes.pop_kind(LexemeKind::Comma);
    }

    Ok(ret)
}
