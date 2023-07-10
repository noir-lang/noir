
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
 
    #[builtin(array_len)]
    fn len(_self: Self) -> comptime Field {}

    #[builtin(arraysort)]
    fn sort(_self: Self) -> Self {}

    // Sort with a custom sorting function.
    fn sort_via(mut a: Self, ordering: fn(T, T) -> bool) -> Self { 
        for i in 1 .. a.len() {
            for j in 0..i {
                if ordering(a[i], a[j]) {
                    let old_a_j = a[j];
                    a[j] = a[i];
                    a[i] = old_a_j;
                }
            }
        }
        a
    }

    // Apply a function to each element of a slice, returning a new slice
    // containing the mapped elements.
    fn map<U>(self, f: fn(T) -> U) -> [U] {
        let mut ret: [U] = [];
        for elem in self {
            ret = ret.push_back(f(elem));
        }
        ret
    }

    // Apply a function to each element of the slice and an accumulator value,
    // returning the final accumulated value. This function is also sometimes
    // called `foldl`, `fold_left`, `reduce`, or `inject`.
    fn fold<U>(self, mut accumulator: U, f: fn(U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    // Apply a function to each element of the slice and an accumulator value,
    // returning the final accumulated value. Unlike fold, reduce uses the first
    // element of the given slice as its starting accumulator value.
    fn reduce(self, f: fn(T, T) -> T) -> T {
        let mut accumulator = self[0];
        for i in 1 .. self.len() {
            accumulator = f(accumulator, self[i]);
        }
        accumulator
    }

    // Returns true if all elements in the array satisfy the predicate
    fn all(self, predicate: fn(T) -> bool) -> bool {
        let mut ret = true;
        for elem in self {
            ret &= predicate(elem);
        }
        ret
    }

    // Returns true if any element in the array satisfies the predicate
    fn any(self, predicate: fn(T) -> bool) -> bool {
        let mut ret = false;
        for elem in self {
            ret |= predicate(elem);
        }
        ret
    }
}

