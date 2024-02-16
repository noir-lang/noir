#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Index(usize);

impl Index {
    #[cfg(test)]
    pub fn test_new(index: usize) -> Index {
        Self(index)
    }

    /// Return a dummy index (max value internally).
    /// This should be avoided over `Option<Index>` if possible.
    pub fn dummy() -> Self {
        Self(usize::MAX)
    }

    /// Return the zeroed index. This is unsafe since we don't know
    /// if this is a valid index for any particular map yet.
    pub fn unsafe_zeroed() -> Self {
        Self(0)
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

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> {
        self.vec.iter().enumerate().map(|(index, item)| (Index(index), item))
    }
}
