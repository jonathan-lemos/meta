#![allow(dead_code)]

use self::Either::*;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn unwrap_left(self) -> L {
        if let Left(l) = self {
            return l;
        }

        panic!("This Either does not contain a Left value.")
    }

    pub fn unwrap_right(self) -> R {
        if let Right(r) = self {
            return r;
        }

        panic!("This Either does not contain a Right value.")
    }

    pub fn map_left<N, F: FnOnce(L) -> N>(self, f: F) -> Either<N, R> {
        match self {
            Left(l) => Left(f(l)),
            Right(r) => Right(r)
        }
    }

    pub fn map_right<N, F: FnOnce(R) -> N>(self, f: F) -> Either<L, N> {
        match self {
            Left(l) => Left(l),
            Right(r) => Right(f(r))
        }
    }

    pub fn left_option(self) -> Option<L> {
        match self {
            Left(l) => Some(l),
            Right(_) => None
        }
    }

    pub fn right_option(self) -> Option<R> {
        match self {
            Left(_) => None,
            Right(r) => Some(r)
        }
    }
}