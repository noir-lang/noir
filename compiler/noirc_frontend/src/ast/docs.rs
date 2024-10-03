use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Documented<T> {
    pub item: T,
    pub doc_comments: Vec<String>,
}

impl<T> Documented<T> {
    pub fn new(item: T, doc_comments: Vec<String>) -> Self {
        Self { item, doc_comments }
    }

    pub fn not_documented(item: T) -> Self {
        Self { item, doc_comments: Vec::new() }
    }
}

impl<T: Display> Display for Documented<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.item)
    }
}
