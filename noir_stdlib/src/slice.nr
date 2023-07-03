
impl<T> [T] {
    /// Push a new element to the end of the slice, returning a
    /// new slice with a length one greater than the
    /// original unmodified vector.
    #[builtin(slice_push_back)]
    fn push_back(_self: Self, _elem: T) -> Self { }

    #[builtin(array_len)]
    fn len(_self: Self) -> comptime Field {}

    #[builtin(arraysort)]
    fn sort(_self: Self) -> Self {}
}

