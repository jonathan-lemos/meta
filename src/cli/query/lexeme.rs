use std::collections::VecDeque;
use crate::linq::collectors::IntoVec;
use crate::cli::query::args::ArgsIter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Lexeme<'a, 'b> {
    token: &'a str,
    kind: LexemeKind,
    cmdline_ptr: &'b str,
}

impl<'a, 'b> Lexeme<'a, 'b> {
    pub fn new(token: &'a str, kind: LexemeKind, cmdline_ptr: &'b str) -> Self {
        Lexeme {token, kind, cmdline_ptr}
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LexemeKind {
    LParen,
    RParen,
    Equals,
    Or,
    And,
    Identifier,
    Value
}

pub struct LexemeQueue<'a, 'b> {
    lexemes: VecDeque<Lexeme<'a, 'b>>
}

impl<'a, 'b> LexemeQueue<'a, 'b> {
    pub fn new() -> Self {
        LexemeQueue { lexemes: VecDeque::new() }
    }

    pub fn from_args_iter(iter: ArgsIter) {
        let ret = Self::new();
        let mut sizes = Vec::<usize>::new();

    }

    pub fn push(&mut self, lexeme: Lexeme<'a, 'b>) {
        self.lexemes.push_back(lexeme);
    }

    pub fn pop(&mut self) -> Option<Lexeme<'a, 'b>> {
        self.lexemes.pop_front()
    }

    pub fn peek(&self) -> Option<&Lexeme<'a, 'b>> {
        self.lexemes.get(0)
    }

    pub fn pop_predicate<F: FnOnce(&Lexeme<'a, 'b>) -> bool>(&mut self, predicate: F) -> Option<Lexeme<'a, 'b>> {
        if predicate(self.peek()?) { self.pop() } else { None }
    }

    pub fn pop_sequence(&mut self, n: usize) -> Option<Vec<Lexeme<'a, 'b>>> {
        if self.lexemes.len() < n {
            return None
        }

        (0..n).map(|_| self.lexemes.pop_front().unwrap()).collect()
    }

    pub fn pop_sequence_predicate<F: FnOnce(&[&Lexeme<'a, 'b>]) -> bool>(&mut self, n: usize, predicate: F) -> Option<Vec<Lexeme<'a, 'b>>> {
        if self.lexemes.len() < n {
            return None
        }

        let seq = (0..n).map(|i| self.lexemes.get(i).unwrap()).into_vec();
        if predicate(&seq[..]) { self.pop_sequence(n) } else { None }
    }
}