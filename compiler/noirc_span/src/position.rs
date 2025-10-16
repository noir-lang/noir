use codespan::Span as ByteSpan;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

pub type Position = u32;

#[derive(PartialOrd, Eq, Ord, Debug, Clone, Default)]
pub struct Spanned<T> {
    pub contents: T,
    span: Span,
}

/// This is important for tests. Two Spanned objects are equal if their content is equal
/// They may not have the same span. Use into_span to test for Span being equal specifically
impl<T: PartialEq> PartialEq<Spanned<T>> for Spanned<T> {
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

    pub fn from(t_span: Span, contents: T) -> Spanned<T> {
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
    PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone, Default, Deserialize, Serialize,
)]
pub struct Span(ByteSpan);

impl Span {
    pub fn inclusive(start: u32, end: u32) -> Span {
        Span(ByteSpan::from(start..end + 1))
    }

    pub fn single_char(start: u32) -> Span {
        Span::inclusive(start, start)
    }

    pub fn empty(position: u32) -> Span {
        Span::from(position..position)
    }

    /// Unlike Span::empty, Span::initial always starts at index 0
    pub const fn initial() -> Span {
        Span(ByteSpan::initial())
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

    pub fn contains(&self, other: &Span) -> bool {
        self.start() <= other.start() && self.end() >= other.end()
    }

    /// Returns `true` if any point of `self` intersects a point of `other`.
    /// Adjacent spans are considered to intersect (so, for example, `0..1` intersects `1..3`).
    pub fn intersects(&self, other: &Span) -> bool {
        self.end() >= other.start() && self.start() <= other.end()
    }

    pub fn is_smaller(&self, other: &Span) -> bool {
        let self_distance = self.end() - self.start();
        let other_distance = other.end() - other.start();
        self_distance < other_distance
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.0.into()
    }
}

impl From<Range<u32>> for Span {
    fn from(Range { start, end }: Range<u32>) -> Self {
        Self(ByteSpan::new(start, end))
    }
}

#[cfg(test)]
mod tests {
    use crate::Span;

    #[test]
    fn test_intersects() {
        assert!(Span::from(5..10).intersects(&Span::from(5..10)));

        assert!(Span::from(5..10).intersects(&Span::from(5..5)));
        assert!(Span::from(5..5).intersects(&Span::from(5..10)));

        assert!(Span::from(10..10).intersects(&Span::from(5..10)));
        assert!(Span::from(5..10).intersects(&Span::from(10..10)));

        assert!(Span::from(5..10).intersects(&Span::from(6..9)));
        assert!(Span::from(6..9).intersects(&Span::from(5..10)));

        assert!(Span::from(5..10).intersects(&Span::from(4..11)));
        assert!(Span::from(4..11).intersects(&Span::from(5..10)));

        assert!(Span::from(5..10).intersects(&Span::from(4..6)));
        assert!(Span::from(4..6).intersects(&Span::from(5..10)));

        assert!(Span::from(5..10).intersects(&Span::from(9..11)));
        assert!(Span::from(9..11).intersects(&Span::from(5..10)));

        assert!(!Span::from(5..10).intersects(&Span::from(3..4)));
        assert!(!Span::from(3..4).intersects(&Span::from(5..10)));

        assert!(!Span::from(5..10).intersects(&Span::from(11..12)));
        assert!(!Span::from(11..12).intersects(&Span::from(5..10)));
    }
}
