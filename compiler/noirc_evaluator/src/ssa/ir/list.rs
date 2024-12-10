use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A shared linked list type intended to be cloned
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct List<T> {
    m: std::marker::PhantomData<T>,
}

// #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// enum Node<T> {
//     #[default]
//     Nil,
//     Cons(T, Arc<Node<T>>),
// }

impl<T> Default for List<T> {
    fn default() -> Self {
        List { m: std::marker::PhantomData }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_back(&mut self, value: T) {
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { m: std::marker::PhantomData }
    }

    pub fn clear(&mut self) {
    }

    pub fn append(&mut self, other: Self)
    where
        T: Copy,
    {
    }

    pub fn len(&self) -> usize {
        0
    }

    pub fn is_empty(&self) -> bool {
        true
    }

    pub fn pop_back(&mut self) -> Option<T>
    where
        T: Default,
    {
        Some(T::default())
    }

    pub fn truncate(&mut self, len: usize)
    where
        T: Copy,
    {
    }

    pub fn unit(item: T) -> Self {
        Self::default()
    }

    pub fn back(&self) -> Option<T> where T: Default {
        Some(T::default())
    }
}

impl<T> Iterator for List<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct Iter<'a, T> {
    m: std::marker::PhantomData<&'a T>,
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<T> std::fmt::Debug for List<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item:?}")?;
        }
        write!(f, "]")
    }
}

impl<T> std::fmt::Display for List<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "]")
    }
}
