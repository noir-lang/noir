#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Index(usize);

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Arena<T> {
    pub vec: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<T> core::ops::Index<Index> for Arena<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.vec.index(index.0)
    }
}

impl<T> core::ops::IndexMut<Index> for Arena<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.vec.index_mut(index.0)
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = T;

    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a T;

    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Index {
        let index = self.vec.len();
        self.vec.push(item);
        Index(index)
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        self.vec.get(index.0)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.vec.get_mut(index.0)
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> {
        self.vec.iter().enumerate().map(|(index, item)| (Index(index), item))
    }
}

/// Side table holding at most one `V` per [`Index`] of an [`Arena`], stored densely by index.
///
/// Use this instead of a hash map when most indices of the arena have an entry.
#[derive(Clone, Debug)]
pub struct ArenaMap<V> {
    vec: Vec<Option<V>>,
}

impl<V> Default for ArenaMap<V> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<V> ArenaMap<V> {
    pub fn insert(&mut self, index: Index, value: V) {
        if index.0 >= self.vec.len() {
            self.vec.resize_with(index.0 + 1, || None);
        }
        self.vec[index.0] = Some(value);
    }

    pub fn get(&self, index: &Index) -> Option<&V> {
        self.vec.get(index.0).and_then(Option::as_ref)
    }

    /// Iterates over the entries in index order.
    pub fn iter(&self) -> impl Iterator<Item = (Index, &V)> {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.as_ref().map(|value| (Index(index), value)))
    }
}
