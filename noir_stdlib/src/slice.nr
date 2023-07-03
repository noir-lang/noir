
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

    // Apply a function to each element of an array, returning a new array
    // containing the mapped elements.
    fn map<U>(self, f: fn(T) -> U) -> [U] {
        let first_elem = f(self[0]);
        let mut ret: [U] = [];
        ret = ret.push_back(first_elem);

        for i in 1 .. self.len() {
            ret.push_back(f(self[i]));
        }

        ret
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. This function is also sometimes
    // called `foldl`, `fold_left`, `reduce`, or `inject`.
    fn fold<U>(self, mut accumulator: U, f: fn(U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. Unlike fold, reduce uses the first
    // element of the given array as its starting accumulator value.
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

