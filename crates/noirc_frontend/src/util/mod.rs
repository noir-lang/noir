/// Equivalent to .into_iter().map(f).collect::<Vec<_>>()
pub fn vecmap<T, U, F>(iterable: T, f: F) -> Vec<U>
where
    T: IntoIterator,
    F: FnMut(T::Item) -> U,
{
    iterable.into_iter().map(f).collect()
}
