use fm::FileId;
use noirc_span::Span;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

pub type Position = u32;

#[derive(Eq, Debug, Clone)]
pub struct Located<T> {
    pub contents: T,
    location: Location,
}

/// This is important for tests. Two Located objects are equal if their content is equal
/// They may not have the same location. Use `.location()` to test for Location being equal specifically
impl<T: PartialEq> PartialEq<Located<T>> for Located<T> {
    fn eq(&self, other: &Located<T>) -> bool {
        self.contents == other.contents
    }
}

impl<T: PartialOrd> PartialOrd<Located<T>> for Located<T> {
    fn partial_cmp(&self, other: &Located<T>) -> Option<Ordering> {
        self.contents.partial_cmp(&other.contents)
    }
}

impl<T: Ord> Ord for Located<T> {
    fn cmp(&self, other: &Located<T>) -> Ordering {
        self.contents.cmp(&other.contents)
    }
}

impl<T: Default> Default for Located<T> {
    fn default() -> Self {
        Self { contents: Default::default(), location: Location::dummy() }
    }
}

/// Hash-based data structures (HashMap, HashSet) rely on the inverse of Hash
/// being injective, i.e. x.eq(y) => hash(x, H) == hash(y, H), we hence align
/// this with the above
impl<T: Hash> Hash for Located<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.contents.hash(state);
    }
}

impl<T> Located<T> {
    pub fn from(location: Location, contents: T) -> Located<T> {
        Located { location, contents }
    }

    pub fn span(&self) -> Span {
        self.location.span
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Location {
    pub span: Span,
    pub file: FileId,
}

impl Location {
    pub fn new(span: Span, file: FileId) -> Self {
        Self { span, file }
    }

    pub const fn dummy() -> Self {
        Self { span: Span::initial(), file: FileId::dummy() }
    }

    pub fn contains(&self, other: &Location) -> bool {
        self.file == other.file && self.span.contains(&other.span)
    }

    #[must_use]
    pub fn merge(self, other: Location) -> Location {
        if self.file == other.file {
            Location::new(self.span.merge(other.span), self.file)
        } else {
            self
        }
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.file, self.span).cmp(&(other.file, other.span))
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
