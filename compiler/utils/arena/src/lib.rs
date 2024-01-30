#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Index {
    pub ix: usize,
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
        self.vec.index(index.ix)
    }
}

impl<T> core::ops::IndexMut<Index> for Arena<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.vec.index_mut(index.ix)
    }
}

impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Index {
        let ix = self.vec.len();
        self.vec.push(item);
        Index { ix }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        self.vec.get(index.ix)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.vec.get_mut(index.ix)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> {
        self.vec.iter().enumerate().map(|(ix, item)| (Index { ix }, item))
    }
}
