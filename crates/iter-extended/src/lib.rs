use std::collections::BTreeMap;

/// Equivalent to .into_iter().map(f).collect::<Vec<_>>()
pub fn vecmap<T, U, F>(iterable: T, f: F) -> Vec<U>
where
    T: IntoIterator,
    F: FnMut(T::Item) -> U,
{
    iterable.into_iter().map(f).collect()
}

/// Equivalent to .into_iter().map(f).collect::<Result<Vec<_>,_>>()
pub fn try_vecmap<T, U, E, F>(iterable: T, f: F) -> Result<Vec<U>, E>
where
    T: IntoIterator,
    F: FnMut(T::Item) -> Result<U, E>,
{
    iterable.into_iter().map(f).collect()
}

/// Equivalent to .into_iter().map(f).collect::<BTreeMap<K, V>>()
pub fn btree_map<T, K, V, F>(iterable: T, f: F) -> BTreeMap<K, V>
where
    T: IntoIterator,
    K: std::cmp::Ord,
    F: FnMut(T::Item) -> (K, V),
{
    iterable.into_iter().map(f).collect()
}
