
// These methods are all stubs currently and aren't implemented internally yet.
// For a similar reason, no constructor for Vec is exposed yet since the type
// is still in-progress.
impl<T> []<T> {
    #[builtin(array_len)]
    fn len(_self: Self) -> comptime Field {}

    /// Push a new element to the end of the slice, returning a
    /// new slice with a length one greater than the
    /// original unmodified vector.
    #[builtin(slice_push_back)]
    fn push_back(_self: Self, _elem: T) -> Self { }
}

