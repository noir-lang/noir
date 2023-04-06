
// These methods are all stubs currently and aren't implemented internally yet.
// For a similar reason, no constructor for Vec is exposed yet since the type
// is still in-progress.
impl<T> Vec<T> {
    /// Get an element from the vector at the given index.
    /// Fails with a constraint error if the given index
    /// points beyond the end of the vector.
    #[builtin(vec_get)]
    fn get(_self: Self, _index: Field) -> T { }

    /// Push a new element to the end of the vector, returning a
    /// new vector with a length one greater than the
    /// original unmodified vector.
    #[builtin(vec_push)]
    fn push(_self: Self, _elem: T) -> Self { }

    /// Pop an element from the end of the given vector, returning
    /// a new vector with a length of one less than the given vector,
    /// as well as the popped element.
    /// Fails with a constraint error if the given vector's length is zero.
    #[builtin(vec_pop)]
    fn pop(_self: Self) -> (Self, T) { }
}
