use codespan::Span as ByteSpan;
use fm::FileId;
use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

pub type Position = u32;

#[derive(PartialOrd, Eq, Ord, Debug, Clone)]
pub struct Spanned<T> {
    pub contents: T,
    span: Span,
}

/// This is important for tests. Two Spanned objects are equal if their content is equal
/// They may not have the same span. Use into_span to test for Span being equal specifically
impl<T: std::cmp::PartialEq> PartialEq<Spanned<T>> for Spanned<T> {
    fn eq(&self, other: &Spanned<T>) -> bool {
        self.contents == other.contents
    }
}

/// Hash-based data structures (HashMap, HashSet) rely on the inverse of Hash
/// being injective, i.e. x.eq(y) => hash(x, H) == hash(y, H), we hence align
/// this with the above
impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.contents.hash(state);
    }
}

impl<T> Spanned<T> {
    pub fn from_position(start: Position, end: Position, contents: T) -> Spanned<T> {
        Spanned { span: Span(ByteSpan::new(start, end)), contents }
    }

    pub const fn from(t_span: Span, contents: T) -> Spanned<T> {
        Spanned { span: t_span, contents }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T> std::borrow::Borrow<T> for Spanned<T> {
    fn borrow(&self) -> &T {
        &self.contents
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone, Default)]
pub struct Span(ByteSpan);

impl Span {
    pub fn new(range: Range<u32>) -> Span {
        Span(ByteSpan::from(range))
    }

    pub fn exclusive(start: u32, end: u32) -> Span {
        Span::new(start..end)
    }

    pub fn inclusive(start: u32, end: u32) -> Span {
        Span::exclusive(start, end + 1)
    }

    pub fn single_char(start: u32) -> Span {
        Span::inclusive(start, start)
    }

    #[must_use]
    pub fn merge(self, other: Span) -> Span {
        Span(self.0.merge(other.0))
    }

    pub fn to_byte_span(self) -> ByteSpan {
        self.0
    }

    pub fn start(&self) -> u32 {
        self.0.start().into()
    }

    pub fn end(&self) -> u32 {
        self.0.end().into()
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.0.into()
    }
}

impl chumsky::Span for Span {
    type Context = ();

    type Offset = u32;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span::new(range)
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start()
    }

    fn end(&self) -> Self::Offset {
        self.end()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub span: Span,
    pub file: FileId,
}

impl Location {
    pub fn new(span: Span, file: FileId) -> Self {
        Self { span, file }
    }
}
