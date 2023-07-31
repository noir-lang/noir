
impl<T> [T] {
    /// Push a new element to the end of the slice, returning a
    /// new slice with a length one greater than the
    /// original unmodified slice.
    #[builtin(slice_push_back)]
    fn push_back(_self: Self, _elem: T) -> Self { }

    /// Push a new element to the front of the slice, returning a
    /// new slice with a length one greater than the 
    /// original unmodified slice. 
    #[builtin(slice_push_front)]
    fn push_front(_self: Self, _elem: T) -> Self { }

    /// Remove the last element of the slice, returning the
    /// popped slice and the element in a tuple
    #[builtin(slice_pop_back)]
    fn pop_back(_self: Self) -> (Self, T) { }

    /// Remove the first element of the slice, returning the
    /// element and the popped slice in a tuple
    #[builtin(slice_pop_front)]
    fn pop_front(_self: Self) -> (T, Self) { }

    /// Insert an element at a specified index, shifting all elements 
    /// after it to the right
    #[builtin(slice_insert)]
    fn insert(_self: Self, _index: Field, _elem: T) -> Self { }

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the altered slice and 
    /// the removed element
    #[builtin(slice_remove)]
    fn remove(_self: Self, _index: Field) -> (Self, T) { }
}

