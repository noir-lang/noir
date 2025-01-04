use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    hash::Hash,
    str::FromStr,
    sync::atomic::{AtomicU32, Ordering},
};
use thiserror::Error;

/// A unique ID corresponding to a value of type T.
/// This type can be used to retrieve a value of type T from
/// either a DenseMap<T> or SparseMap<T>.
///
/// Note that there is nothing in an Id binding it to a particular
/// DenseMap or SparseMap. If an Id was created to correspond to one
/// particular map type, users need to take care not to use it with
/// another map where it will likely be invalid.
#[derive(Serialize, Deserialize)]
pub(crate) struct Id<T> {
    index: u32,
    // If we do not skip this field it will simply serialize as `"_marker":null` which is useless extra data
    #[serde(skip)]
    _marker: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    /// Constructs a new Id for the given index.
    ///
    /// This is private so that we can guarantee ids created from this function
    /// point to valid T values in their external maps.
    fn new(index: u32) -> Self {
        Self { index, _marker: std::marker::PhantomData }
    }

    /// Returns the underlying index of this Id.
    pub(crate) fn to_u32(self) -> u32 {
        self.index
    }

    /// Creates a test Id with the given index.
    /// The name of this function makes it apparent it should only
    /// be used for testing. Obtaining Ids in this way should be avoided
    /// as unlike DenseMap::push and SparseMap::push, the Ids created
    /// here are likely invalid for any particularly map.
    #[cfg(test)]
    pub(crate) fn test_new(index: u32) -> Self {
        Self::new(index)
    }
}

// Need to manually implement most impls on Id.
// Otherwise rust assumes that Id<T>: Hash only if T: Hash,
// which isn't true since the T is not used internally.
impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Deliberately formatting as a tuple with 1 element here and omitting
        // the _marker: PhantomData field which would just clutter output
        f.debug_tuple("Id").field(&self.index).finish()
    }
}

impl std::fmt::Display for Id<super::basic_block::BasicBlock> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.index)
    }
}

impl std::fmt::Display for Id<super::value::Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.index)
    }
}

impl std::fmt::Display for Id<super::function::Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.index)
    }
}

impl std::fmt::Display for Id<super::instruction::Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i{}", self.index)
    }
}

#[derive(Error, Debug)]
pub(crate) enum IdDisplayFromStrErr {
    #[error("Invalid id when deserializing SSA: {0}")]
    InvalidId(String),
}

/// The implementation of display and FromStr allows serializing and deserializing an Id<T> to a string.
/// This is useful when used as key in a map that has to be serialized to JSON/TOML.
impl FromStr for Id<super::basic_block::BasicBlock> {
    type Err = IdDisplayFromStrErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        id_from_str_helper::<super::basic_block::BasicBlock>(s, 'b')
    }
}

impl FromStr for Id<super::value::Value> {
    type Err = IdDisplayFromStrErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        id_from_str_helper::<super::value::Value>(s, 'v')
    }
}

impl FromStr for Id<super::function::Function> {
    type Err = IdDisplayFromStrErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        id_from_str_helper::<super::function::Function>(s, 'f')
    }
}

impl FromStr for Id<super::instruction::Instruction> {
    type Err = IdDisplayFromStrErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        id_from_str_helper::<super::instruction::Instruction>(s, 'i')
    }
}

fn id_from_str_helper<T>(s: &str, value_prefix: char) -> Result<Id<T>, IdDisplayFromStrErr> {
    if s.len() < 2 {
        return Err(IdDisplayFromStrErr::InvalidId(s.to_string()));
    }

    let index = &s[1..];
    let index = index.parse().map_err(|_| IdDisplayFromStrErr::InvalidId(s.to_string()))?;

    if s.chars().next().unwrap() == value_prefix {
        Ok(Id::<T>::new(index))
    } else {
        Err(IdDisplayFromStrErr::InvalidId(s.to_string()))
    }
}

/// A DenseMap is a Vec wrapper where each element corresponds
/// to a unique ID that can be used to access the element. No direct
/// access to indices is provided. Since IDs must be stable and correspond
/// to indices in the internal Vec, operations that would change element
/// ordering like pop, remove, swap_remove, etc, are not possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DenseMap<T> {
    storage: Vec<T>,
}

impl<T> DenseMap<T> {
    /// Adds an element to the map.
    /// Returns the identifier/reference to that element.
    pub(crate) fn insert(&mut self, element: T) -> Id<T> {
        let id = Id::new(self.storage.len().try_into().unwrap());
        self.storage.push(element);
        id
    }

    /// Gets an iterator to a reference to each element in the dense map paired with its id.
    ///
    /// The id-element pairs are ordered by the numeric values of the ids.
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = (Id<T>, &T)> {
        let ids_iter = (0..self.storage.len() as u32).map(|idx| Id::new(idx));
        ids_iter.zip(self.storage.iter())
    }
}

impl<T> Default for DenseMap<T> {
    fn default() -> Self {
        Self { storage: Vec::new() }
    }
}

impl<T> std::ops::Index<Id<T>> for DenseMap<T> {
    type Output = T;

    fn index(&self, id: Id<T>) -> &Self::Output {
        &self.storage[id.index as usize]
    }
}

impl<T> std::ops::IndexMut<Id<T>> for DenseMap<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut Self::Output {
        &mut self.storage[id.index as usize]
    }
}

/// A SparseMap is a HashMap wrapper where each element corresponds
/// to a unique ID that can be used to access the element. No direct
/// access to indices is provided.
///
/// Unlike DenseMap, SparseMap's IDs are stored within the structure
/// and are thus stable after element removal.
///
/// Note that unlike DenseMap, it is possible to panic when retrieving
/// an element if the element's Id has been invalidated by a previous
/// call to .remove().
#[derive(Debug)]
pub(crate) struct SparseMap<T> {
    storage: BTreeMap<Id<T>, T>,
}

impl<T> SparseMap<T> {
    /// Given the Id of the element being created, adds the element
    /// returned by the given function to the map
    pub(crate) fn insert_with_id(&mut self, f: impl FnOnce(Id<T>) -> T) -> Id<T> {
        let id = Id::new(self.storage.len().try_into().unwrap());
        self.storage.insert(id, f(id));
        id
    }

    /// Unwraps the inner storage of this map
    pub(crate) fn into_btree(self) -> BTreeMap<Id<T>, T> {
        self.storage
    }
}

impl<T> Default for SparseMap<T> {
    fn default() -> Self {
        Self { storage: Default::default() }
    }
}

impl<T> std::ops::Index<Id<T>> for SparseMap<T> {
    type Output = T;

    fn index(&self, id: Id<T>) -> &Self::Output {
        &self.storage[&id]
    }
}

impl<T> std::ops::IndexMut<Id<T>> for SparseMap<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut Self::Output {
        self.storage.get_mut(&id).expect("Invalid id used in SparseMap::index_mut")
    }
}

/// A simple counter to create fresh Ids without any storage.
/// Useful for assigning ids before the storage is created or assigning ids
/// for types that have no single owner.
///
/// This type wraps an AtomicUsize so it can safely be used across threads.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AtomicCounter<T> {
    next: AtomicU32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> AtomicCounter<T> {
    /// Create a new counter starting after the given Id.
    /// Use AtomicCounter::default() to start at zero.
    pub(crate) fn starting_after(id: Id<T>) -> Self {
        Self { next: AtomicU32::new(id.index + 1), _marker: Default::default() }
    }

    /// Return the next fresh id
    pub(crate) fn next(&self) -> Id<T> {
        Id::new(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

impl<T> Default for AtomicCounter<T> {
    fn default() -> Self {
        Self { next: Default::default(), _marker: Default::default() }
    }
}
