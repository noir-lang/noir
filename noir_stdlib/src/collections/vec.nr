
// These methods are all stubs currently and aren't implemented internally yet.
// For a similar reason, no constructor for Vec is exposed yet since the type
// is still in-progress.
impl<T> Vec<T> {
    #[builtin(vec_get)]
    fn get(_self: Self, _index: Field) -> T { }

    #[builtin(vec_push)]
    fn push(_self: Self, _elem: T) -> Self { }

    #[builtin(vec_pop)]
    fn pop(_self: Self) -> (Self, T) { }
}
