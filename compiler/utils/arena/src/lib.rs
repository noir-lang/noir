#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
// pub struct Index(pub usize);
pub struct Index(pub usize, pub generational_arena::Index);

// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
#[derive(Debug, Clone)]
pub struct Arena<T> {
    pub vec: Vec<T>,
    pub _arena: generational_arena::Arena<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            vec: Vec::new(),
            _arena: generational_arena::Arena::default(),
        }
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

// TODO: remove Clone
impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Index
    where
        T: Clone,
    {
        let index = self.vec.len();
        self.vec.push(item.clone());
        let index_arena = self._arena.insert(item);
        Index(index, index_arena)
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        self.vec.get(index.0)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.vec.get_mut(index.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> {
        self.vec
            .iter()
            .enumerate()
            .map(|(index, item)| (Index(index, generational_arena::Index::from_raw_parts(index, 0)), item))
    }

}

