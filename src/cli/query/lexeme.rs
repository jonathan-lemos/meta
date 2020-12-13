use std::collections::VecDeque;

use crate::cli::query::args::ArgsIter;
use crate::linq::collectors::IntoVec;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct OwnedLexeme {
    pub token: String,
    pub kind: LexemeKind,
    pub cmdline: String,
    pub cmdline_index: usize
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Lexeme<'a, 'b> {
    token: &'a str,
    kind: LexemeKind,
    cmdline_ptr: &'b str,
    cmdline_index: usize
}

impl<'a, 'b> ToOwned for Lexeme<'a, 'b> {
    type Owned = OwnedLexeme;

    fn to_owned(&self) -> Self::Owned {
        OwnedLexeme {
            token: self.token.to_owned(),
            kind: self.kind,
            cmdline: self.cmdline_ptr.to_owned(),
            cmdline_index: self.cmdline_index
        }
    }
}

impl<'a, 'b> Lexeme<'a, 'b> {
    pub fn new(token: &'a str, kind: LexemeKind, cmdline_ptr: &'b str, cmdline_index: usize) -> Self {
        Lexeme { token, kind, cmdline_ptr, cmdline_index }
    }

    pub fn token(&self) -> &str {
        self.token
    }

    pub fn kind(&self) -> LexemeKind {
        self.kind
    }

    pub fn cmdline(&self) -> &str {
        self.cmdline_ptr
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LexemeKind {
    LParen,
    RParen,
    Equals(EqualityKind),
    Or,
    And,
    Key,
    Value,
    In,
    Comma,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum EqualityKind {
    Strict,
    Matches,
}

pub struct LexemeQueue<'a, 'b> {
    lexemes: VecDeque<Lexeme<'a, 'b>>
}

impl<'a, 'b> LexemeQueue<'a, 'b> {
    pub fn new() -> Self {
        LexemeQueue { lexemes: VecDeque::new() }
    }

    pub fn len(&self) -> usize {
        self.lexemes.len()
    }

    pub fn peek(&self) -> Option<&Lexeme<'a, 'b>> {
        self.lexemes.get(0)
    }

    pub fn pop(&mut self) -> Option<Lexeme<'a, 'b>> {
        self.lexemes.pop_front()
    }

    pub fn pop_kind(&mut self, kind: LexemeKind) -> Option<Lexeme<'a, 'b>> {
        self.pop_predicate(|l| l.kind() == kind)
    }

    pub fn pop_predicate<F: FnOnce(&Lexeme<'a, 'b>) -> bool>(&mut self, predicate: F) -> Option<Lexeme<'a, 'b>> {
        if predicate(self.peek()?) { self.pop() } else { None }
    }

    pub fn pop_sequence(&mut self, n: usize) -> Option<Vec<Lexeme<'a, 'b>>> {
        if self.lexemes.len() < n {
            return None;
        }

        (0..n).map(|_| self.lexemes.pop_front().unwrap()).collect()
    }

    pub fn pop_sequence_predicate<F: FnOnce(&[&Lexeme<'a, 'b>]) -> bool>(&mut self, n: usize, predicate: F) -> Option<Vec<Lexeme<'a, 'b>>> {
        if self.lexemes.len() < n {
            return None;
        }

        let seq = (0..n).map(|i| self.lexemes.get(i).unwrap()).into_vec();
        if predicate(&seq[..]) { self.pop_sequence(n) } else { None }
    }

    pub fn push(&mut self, lexeme: Lexeme<'a, 'b>) {
        self.lexemes.push_back(lexeme);
    }
}