#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Index {
    pub index: usize,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Arena<T> {
    vec: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<T> core::ops::Index<Index> for Arena<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.vec.index(index.index)
    }
}

impl<T> core::ops::IndexMut<Index> for Arena<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.vec.index_mut(index.index)
    }
}

impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Index {
        let index = self.vec.len();
        self.vec.push(item);
        Index { index }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        self.vec.get(index.index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.vec.get_mut(index.index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> {
        self.vec.iter().enumerate().map(|(index, item)| (Index { index }, item))
    }
}
