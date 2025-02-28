#![feature(iter_next_chunk)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    ops::{BitOr, Bound::*, RangeBounds, Shl, ShlAssign, Shr, ShrAssign},
};

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait TokenStream<T: AbcToken> {
    fn parse<P: Parse<Token = T> + Sized>(&mut self) -> Result<P, P::Err> {
        P::parse(self)
    }

    fn peek<P: Peek<TokenType = T::TokenType> + Sized>(&self) -> bool {
        if let Some(ty) = self.peek_token() {
            ty.token_type() == P::token_type()
        }
        else {
            false
        }
    }

    fn peek2<P: Peek<TokenType = T::TokenType> + Sized>(&self) -> bool {
        if let Some(ty) = self.peek2_token() {
            ty.token_type() == P::token_type()
        }
        else {
            false
        }
    }

    fn is_end(&self) -> bool {
        self.peek_token().is_some()
    }

    fn peek_token(&self) -> Option<&T>;

    fn peek2_token(&self) -> Option<&T>;

    fn next_token(&mut self) -> Option<T>;
}

pub trait Parse: Sized {
    type Token: AbcToken;
    type Err;

    fn parse<TS: TokenStream<Self::Token> + ?Sized>(
        input: &mut TS,
    ) -> Result<Self, Self::Err>;
}

pub trait Peek {
    type TokenType: Eq;

    fn token_type() -> Self::TokenType;
}

pub trait AbcToken {
    type TokenType: Eq;

    fn token_type(&self) -> Self::TokenType;
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: isize,
    /// exclusive
    pub end: isize,
}

pub struct SliceTokenStream<'t, T> {
    slice: &'t [T],
    ptr: usize,
}

pub struct IteratorTokenStream<I, T> {
    iter: I,
    queue: VecDeque<T>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl Span {
    pub fn offset(&self, offset: isize) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
    pub fn union(it:Self, oth: Self) -> Self {
        Self { start: it.start.min(oth.start), end: it.end.max(oth.end) }
    }
}

impl BitOr<Self> for Span {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::union(self, rhs)
    }
}

impl Shl<usize> for Span {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self {
            start: self.start - rhs as isize,
            end: self.end - rhs as isize ,
        }
    }
}

impl Shr<usize> for Span {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self {
            start: self.start + rhs as isize,
            end: self.end + rhs as isize
        }
    }
}

impl ShlAssign<usize> for Span {
    fn shl_assign(&mut self, rhs: usize) {
        self.start -= rhs as isize;
        self.end -= rhs as isize
    }
}

impl ShrAssign<usize> for Span {
    fn shr_assign(&mut self, rhs: usize) {
        self.start += rhs as isize;
        self.end += rhs as isize;
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<T: RangeBounds<usize>> From<T> for Span {
    fn from(value: T) -> Self {
        let start = match value.start_bound() {
            Included(&v) => v as _,
            Excluded(&v) => v as isize - 1,
            Unbounded => 0,
        };

        let end = match value.end_bound() {
            Included(&v) => v as isize + 1,
            Excluded(&v) => v as _,
            Unbounded => unimplemented!(),
        };

        Self { start, end }
    }
}


impl<I: Iterator<Item = T>, T: AbcToken> TokenStream<T>
    for IteratorTokenStream<I, T>
{
    fn peek_token(&self) -> Option<&T> {
        self.queue.front()
    }

    fn peek2_token(&self) -> Option<&T> {
        if self.queue.len() >= 2 {
            Some(&self.queue[1])
        }
        else {
            None
        }
    }

    fn next_token(&mut self) -> Option<T> {
        if let Some(v) = self.iter.next() {
            self.queue.push_back(v);
        }

        self.queue.pop_front()
    }
}

impl<I: Iterator<Item = T>, T: AbcToken> From<I>
    for IteratorTokenStream<I, T>
{
    fn from(mut iter: I) -> Self {
        let mut queue = VecDeque::with_capacity(2);

        match iter.next_chunk::<2>() {
            Ok(arr) => queue.extend(arr.into_iter()), // push_back
            Err(intoiter) => queue.extend(intoiter.into_iter()),
        }

        Self { iter, queue }
    }
}

impl<'t, T: AbcToken + Clone> TokenStream<T> for SliceTokenStream<'t, T> {
    fn peek_token(&self) -> Option<&T> {
        if self.ptr < self.slice.len() {
            Some(&self.slice[self.ptr])
        }
        else {
            None
        }
    }

    fn peek2_token(&self) -> Option<&T> {
        if self.ptr + 1 < self.slice.len() {
            Some(&self.slice[self.ptr + 1])
        }
        else {
            None
        }
    }

    fn next_token(&mut self) -> Option<T> {
        if self.ptr < self.slice.len() {
            Some(self.slice[self.ptr].clone())
        }
        else {
            None
        }
    }
}


impl<'t, T> From<&'t [T]> for SliceTokenStream<'t, T> {
    fn from(value: &'t [T]) -> Self {
        Self {
            slice: value,
            ptr: 0,
        }
    }
}



#[cfg(test)]
mod tests {}
