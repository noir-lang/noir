use std::collections::BTreeMap;

/// Equivalent to .into_iter().map(f).collect::<Vec<_>>()
pub fn vecmap<T, U, F>(iterable: T, f: F) -> Vec<U>
where
    T: IntoIterator,
    F: FnMut(T::Item) -> U,
{
    iterable.into_iter().map(f).collect()
}

/// Equivalent to .into_iter().map(f).collect()
pub fn btree_map<T, K, V, F>(iterable: T, f: F) -> BTreeMap<K, V>
where
    T: IntoIterator,
    K: std::cmp::Ord,
    F: FnMut(T::Item) -> (K, V),
{
    iterable.into_iter().map(f).collect()
}
