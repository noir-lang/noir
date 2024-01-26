use codespan::Span as ByteSpan;

use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

#[derive(Copy, Clone, Default)]
pub struct Position(u32, SrcId);

impl Position {
    pub fn new(pos: u32, src_id: SrcId) -> Position {
        Position(pos, src_id)
    }

    pub fn src_id(&self) -> SrcId {
        self.1
    }
}
impl std::ops::Add<u32> for Position {
    type Output = Position;

    fn add(self, rhs: u32) -> Position {
        Position(self.0 + rhs, self.1)
    }
}

impl From<Position> for u32 {
    fn from(val: Position) -> Self {
        val.0
    }
}

impl From<Position> for Span {
    fn from(val: Position) -> Self {
        Span::single_char(val.0, val.1)
    }
}

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
        Spanned { span: Span::inclusive(start, end), contents }
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

#[derive(
    Copy, Clone, Serialize, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Hash, Default,
)]
pub struct SrcId(pub usize);

impl From<usize> for SrcId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<SrcId> for usize {
    fn from(val: SrcId) -> Self {
        val.0
    }
}

#[derive(
    PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone, Default, Deserialize, Serialize,
)]
pub struct Span(ByteSpan, SrcId);

impl Span {
    pub fn inclusive(start: Position, end: Position) -> Span {
        Span(ByteSpan::from(start.0..end.0 + 1), start.1)
    }

    pub fn inclusive_within(start: u32, end: u32, src_id: impl Into<usize>) -> Span {
        Span(ByteSpan::from(start..end + 1), src_id.into().into())
    }

    pub fn single_char(start: u32, src_id: impl Into<usize>) -> Span {
        let start_position = Position::new(start, src_id.into().into());
        Span::inclusive(start_position, start_position)
    }

    pub fn empty(position: u32, src_id: SrcId) -> Span {
        Span::from_range(position..position, src_id)
    }

    #[must_use]
    pub fn merge(self, other: Span) -> Span {
        Span(self.0.merge(other.0), self.1)
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

    pub fn contains(&self, other: &Span) -> bool {
        self.start() <= other.start() && self.end() >= other.end()
    }

    pub fn is_smaller(&self, other: &Span) -> bool {
        let self_distance = self.end() - self.start();
        let other_distance = other.end() - other.start();
        self_distance < other_distance
    }

    fn from_range(Range { start, end }: Range<u32>, src_id: SrcId) -> Self {
        Self(ByteSpan::new(start, end), src_id)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.0.into()
    }
}

impl From<Range<u32>> for Span {
    fn from(Range { start, end }: Range<u32>) -> Self {
        Self(ByteSpan::new(start, end), SrcId::default())
    }
}

impl chumsky::Span for Span {
    type Context = SrcId;

    type Offset = u32;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span(ByteSpan::from(range), context)
    }

    fn context(&self) -> Self::Context {
        self.1
    }

    fn start(&self) -> Self::Offset {
        self.start()
    }

    fn end(&self) -> Self::Offset {
        self.end()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Location {
    pub span: Span,
    pub file: SrcId,
}

impl Location {
    pub fn new(span: Span, file: SrcId) -> Self {
        Self { span, file }
    }

    pub fn dummy() -> Self {
        let file = SrcId::default();
        Self { span: Span::single_char(0, file), file }
    }

    pub fn contains(&self, other: &Location) -> bool {
        self.file == other.file && self.span.contains(&other.span)
    }
}
