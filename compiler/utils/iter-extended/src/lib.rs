#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

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

/// Equivalent to .into_iter().map(f).collect::<Result<BTreeMap<_, _>,_>>()
pub fn try_btree_map<T, K, V, E, F>(iterable: T, f: F) -> Result<BTreeMap<K, V>, E>
where
    T: IntoIterator,
    K: std::cmp::Ord,
    F: FnMut(T::Item) -> Result<(K, V), E>,
{
    iterable.into_iter().map(f).collect()
}

/// Given an iterator over a Result, filter out the Ok values from the Err values
/// and return both in separate Vecs. Unlike other collect-like functions over Results,
/// this function will always consume the entire iterator.
pub fn partition_results<It, T, E, F>(iterable: It, mut f: F) -> (Vec<T>, Vec<E>)
where
    It: IntoIterator,
    F: FnMut(It::Item) -> Result<T, E>,
{
    let mut oks = vec![];
    let mut errors = vec![];

    for elem in iterable {
        match f(elem) {
            Ok(ok) => oks.push(ok),
            Err(error) => errors.push(error),
        }
    }

    (oks, errors)
}
