use codespan::Span as ByteSpan;

use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

#[derive(Copy, Clone, Default)]
pub struct Position(u32, SrcId);

impl Position {
    pub fn new(offset: u32, src_id: SrcId) -> Position {
        Position(offset, src_id)
    }

    pub fn src_id(&self) -> SrcId {
        self.1
    }

    pub fn offset(&self) -> u32 {
        self.0
    }
}
impl std::ops::Add<u32> for Position {
    type Output = Position;

    fn add(self, rhs: u32) -> Position {
        Position(self.0 + rhs, self.1)
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

/// [SrcId] represents id which particualr source originated from
/// ie. a file in context of file system or url in context of web, etc.
/// [SrcId] is used as conext for [Span] and [Position] which originate
/// [SrcId] is used to uniquely identify a source within [fm::FileMap]
/// from a resource.
#[derive(
    Copy, Clone, Serialize, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Hash, Default,
)]
pub struct SrcId(pub usize);

impl From<SrcId> for usize {
    fn from(val: SrcId) -> Self {
        val.0
    }
}

#[derive(
    PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone, Default, Deserialize, Serialize,
)]
pub struct Span {
    byte_span: ByteSpan,
    src_id: SrcId,
}

impl Span {
    pub fn inclusive(start: Position, end: Position) -> Span {
        Span { byte_span: ByteSpan::from(start.0..end.0 + 1), src_id: start.1 }
    }

    pub fn inclusive_within(start: u32, end: u32, src_id: SrcId) -> Span {
        Span { byte_span: ByteSpan::from(start..end + 1), src_id }
    }

    pub fn single_char(start: u32, src_id: SrcId) -> Span {
        let start_position = Position::new(start, src_id);
        Span::inclusive(start_position, start_position)
    }

    pub fn empty(position: u32, src_id: SrcId) -> Span {
        Span::from_range(position..position, src_id)
    }

    #[must_use]
    pub fn merge(self, other: Span) -> Span {
        Span { byte_span: self.byte_span.merge(other.byte_span), src_id: self.src_id }
    }

    pub fn to_byte_span(self) -> ByteSpan {
        self.byte_span
    }

    pub fn start(&self) -> u32 {
        self.byte_span.start().into()
    }

    pub fn end(&self) -> u32 {
        self.byte_span.end().into()
    }

    pub fn contains(&self, other: &Span) -> bool {
        self.src_id == other.src_id && self.start() <= other.start() && self.end() >= other.end()
    }

    pub fn is_smaller(&self, other: &Span) -> bool {
        let self_distance = self.end() - self.start();
        let other_distance = other.end() - other.start();
        self_distance < other_distance
    }

    fn from_range(Range { start, end }: Range<u32>, src_id: SrcId) -> Self {
        Self { byte_span: ByteSpan::new(start, end), src_id: src_id }
    }

    pub fn src_id(&self) -> SrcId {
        self.src_id
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.byte_span.into()
    }
}

impl chumsky::Span for Span {
    type Context = SrcId;

    type Offset = u32;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span { byte_span: ByteSpan::from(range), src_id: context }
    }

    fn context(&self) -> Self::Context {
        self.src_id
    }

    fn start(&self) -> Self::Offset {
        self.start()
    }

    fn end(&self) -> Self::Offset {
        self.end()
    }
}
