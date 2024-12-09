use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A shared linked list type intended to be cloned
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct List<T> {
    head: Arc<Node<T>>,
    len: usize,
}

#[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Node<T> {
    #[default]
    Nil,
    Cons(T, Arc<Node<T>>),
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List { head: Arc::new(Node::Nil), len: 0 }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_back(&mut self, value: T) {
        self.len += 1;
        self.head = Arc::new(Node::Cons(value, self.head.clone()));
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { head: &self.head, len: self.len }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn append(&mut self, other: Self)
    where
        T: Copy,
    {
        let other = other.collect::<Vec<_>>();

        for item in other.into_iter().rev() {
            self.push_back(item);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn pop_back(&mut self) -> Option<T>
    where
        T: Copy,
    {
        match self.head.as_ref() {
            Node::Nil => None,
            Node::Cons(value, rest) => {
                let value = *value;
                self.head = rest.clone();
                self.len -= 1;
                Some(value)
            }
        }
    }

    pub fn truncate(&mut self, len: usize)
    where
        T: Copy,
    {
        if self.len > len {
            for _ in 0..self.len - len {
                self.pop_back();
            }
        }
    }

    pub fn unit(item: T) -> Self {
        let mut this = Self::default();
        this.push_back(item);
        this
    }

    pub fn back(&self) -> Option<&T> {
        match self.head.as_ref() {
            Node::Nil => None,
            Node::Cons(item, _) => Some(item),
        }
    }
}

impl<T> Iterator for List<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_back()
    }
}

pub struct Iter<'a, T> {
    head: &'a Node<T>,
    len: usize,
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
        match self.head {
            Node::Nil => None,
            Node::Cons(value, rest) => {
                self.head = rest;
                Some(value)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
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
