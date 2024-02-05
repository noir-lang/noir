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


impl<T> core::ops::Index<Index> for Arena<T>
// TODO: remove
where
    T: PartialEq + std::fmt::Debug,
{
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        // self.vec.index(index.0)
        let vec_out = self.vec.index(index.0);
        let arena_out = self._arena.index(index.1);

        if vec_out == arena_out {
            vec_out
        } else {
            panic!("Arena::index: vec_out != arena_out:\n Index: {:?}\n \n {:?} != {:?}", index, vec_out, arena_out)
        }
    }
}

impl<T> core::ops::IndexMut<Index> for Arena<T>
// TODO: remove
where
    T: PartialEq + std::fmt::Debug,
{
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        // self.vec.index_mut(index.0)
        let vec_out = self.vec.get(index.0);
        let arena_out = self._arena.get(index.1);

        if vec_out == arena_out {
            self.vec.index_mut(index.0)
        } else {
            let vec_iter = self.vec.iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join("\n\n");
            let arena_iter = self._arena.iter().map(|x| format!("{:?}", x.1)).collect::<Vec<_>>().join("\n\n");
            panic!("Arena::index_mut: vec_out != arena_out:\n Index: {:?}\n \n {:?}\n !=\n {:?}\n\n Vec:\n{}\n\n Arena:\n{}", index, self.vec.get(index.0), self._arena.get(index.1), vec_iter, arena_iter)
        }
    }
}

// TODO: remove Clone
impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Index
    where
        T: Clone + std::fmt::Debug,
    {
        let index = self.vec.len();
        self.vec.push(item.clone());

        let index_arena = self._arena.insert(item);
        if index == index_arena.into_raw_parts().0 {
            Index(index, index_arena)
        } else {
            panic!("Arena::insert: index != index_arena: {:?} != {:?}", index, index_arena)
        }

        // Index(index)
    }

    pub fn get(&self, index: Index) -> Option<&T>
    where
        T: PartialEq + std::fmt::Debug,
    {
        // self.vec.get(index.0)
        let item = self.vec.get(index.0);
        let item_arena = self._arena.get(index.1);
        if item == item_arena {
            item
        } else {
            panic!("Arena::get: {:?} != {:?}", item, item_arena)
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    where
        T: PartialEq + std::fmt::Debug,
    {
        // self.vec.get_mut(index.0)
        let item = self.vec.get_mut(index.0);
        let item_arena = self._arena.get_mut(index.1);
        if item == item_arena {
            item
        } else {
            panic!("Arena::get: {:?} != {:?}", item, item_arena)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)>
    where
        T: PartialEq + std::fmt::Debug,
    {
        // self.vec.iter().enumerate().map(|(index, item)| (Index(index), item))
        self.vec
            .iter()
            .enumerate()
            .map(|(index, item)| (Index(index, generational_arena::Index::from_raw_parts(index, 0)), item))
            .zip(self._arena.iter())
            .map(|(item, item_arena)| {
                if item.0.0 == item_arena.0.into_raw_parts().0 && item.1 == item_arena.1 {
                    Ok(item)
                } else {
                    Err(format!("Arena::iter: {:?} != {:?}", item, item_arena))
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
    }

}

