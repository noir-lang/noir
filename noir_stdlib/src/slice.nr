use crate::append::Append;

impl<T> [T] {
    /// Returns the length of the slice.
    #[builtin(array_len)]
    pub fn len(self) -> u32 {}

    /// Push a new element to the end of the slice, returning a
    /// new slice with a length one greater than the
    /// original unmodified slice.
    #[builtin(slice_push_back)]
    pub fn push_back(self, elem: T) -> Self {}

    /// Push a new element to the front of the slice, returning a
    /// new slice with a length one greater than the
    /// original unmodified slice.
    #[builtin(slice_push_front)]
    pub fn push_front(self, elem: T) -> Self {}

    /// Remove the last element of the slice, returning the
    /// popped slice and the element in a tuple
    #[builtin(slice_pop_back)]
    pub fn pop_back(self) -> (Self, T) {}

    /// Remove the first element of the slice, returning the
    /// element and the popped slice in a tuple
    #[builtin(slice_pop_front)]
    pub fn pop_front(self) -> (T, Self) {}

    /// Insert an element at a specified index, shifting all elements
    /// after it to the right
    #[builtin(slice_insert)]
    pub fn insert(self, index: u32, elem: T) -> Self {}

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the altered slice and
    /// the removed element
    #[builtin(slice_remove)]
    pub fn remove(self, index: u32) -> (Self, T) {}

    /// Append each element of the `other` slice to the end of `self`.
    /// This returns a new slice and leaves both input slices unchanged.
    pub fn append(mut self, other: Self) -> Self {
        for elem in other {
            self = self.push_back(elem);
        }
        self
    }

    pub fn as_array<let N: u32>(self) -> [T; N] {
        assert(self.len() == N);

        let mut array = [crate::mem::zeroed(); N];
        for i in 0..N {
            array[i] = self[i];
        }
        array
    }

    // Apply a function to each element of the slice, returning a new slice
    // containing the mapped elements.
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U] {
        let mut ret = &[];
        for elem in self {
            ret = ret.push_back(f(elem));
        }
        ret
    }

    // Apply a function to each element of the slice with its index, returning a
    // new slice containing the mapped elements.
    pub fn mapi<U, Env>(self, f: fn[Env](u32, T) -> U) -> [U] {
        let mut ret = &[];
        let mut index = 0;
        for elem in self {
            ret = ret.push_back(f(index, elem));
            index += 1;
        }
        ret
    }

    // Apply a function to each element of the slice
    pub fn for_each<Env>(self, f: fn[Env](T) -> ()) {
        for elem in self {
            f(elem);
        }
    }

    // Apply a function to each element of the slice with its index
    pub fn for_eachi<Env>(self, f: fn[Env](u32, T) -> ()) {
        let mut index = 0;
        for elem in self {
            f(index, elem);
            index += 1;
        }
    }

    // Apply a function to each element of the slice and an accumulator value,
    // returning the final accumulated value. This function is also sometimes
    // called `foldl`, `fold_left`, `reduce`, or `inject`.
    pub fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    // Apply a function to each element of the slice and an accumulator value,
    // returning the final accumulated value. Unlike fold, reduce uses the first
    // element of the given slice as its starting accumulator value.
    pub fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T {
        let mut accumulator = self[0];
        for i in 1..self.len() {
            accumulator = f(accumulator, self[i]);
        }
        accumulator
    }

    // Returns a new slice containing only elements for which the given predicate
    // returns true.
    pub fn filter<Env>(self, predicate: fn[Env](T) -> bool) -> Self {
        let mut ret = &[];
        for elem in self {
            if predicate(elem) {
                ret = ret.push_back(elem);
            }
        }
        ret
    }

    // Flatten each element in the slice into one value, separated by `separator`.
    pub fn join(self, separator: T) -> T
    where
        T: Append,
    {
        let mut ret = T::empty();

        if self.len() != 0 {
            ret = self[0];

            for i in 1..self.len() {
                ret = ret.append(separator).append(self[i]);
            }
        }

        ret
    }

    // Returns true if all elements in the slice satisfy the predicate
    pub fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = true;
        for elem in self {
            ret &= predicate(elem);
        }
        ret
    }

    // Returns true if any element in the slice satisfies the predicate
    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        for elem in self {
            ret |= predicate(elem);
        }
        ret
    }
}

mod test {
    #[test]
    fn map_empty() {
        assert_eq(&[].map(|x| x + 1), &[]);
    }

    #[test]
    fn mapi_empty() {
        assert_eq(&[].mapi(|i, x| i * x + 1), &[]);
    }

    #[test]
    fn for_each_empty() {
        let empty_slice: [Field] = &[];
        empty_slice.for_each(|_x| assert(false));
        assert(empty_slice == &[]);
    }

    #[test]
    fn for_eachi_empty() {
        let empty_slice: [Field] = &[];
        empty_slice.for_eachi(|_i, _x| assert(false));
        assert(empty_slice == &[]);
    }

    #[test]
    fn map_example() {
        let a = &[1, 2, 3];
        let b = a.map(|a| a * 2);
        assert_eq(b, &[2, 4, 6]);
        assert_eq(a, &[1, 2, 3]);
    }

    #[test]
    fn mapi_example() {
        let a = &[1, 2, 3];
        let b = a.mapi(|i, a| i + a * 2);
        assert_eq(b, &[2, 5, 8]);
        assert_eq(a, &[1, 2, 3]);
    }

    #[test]
    fn for_each_example() {
        let a = &[1, 2, 3];
        let mut b = &[];
        let b_ref = &mut b;
        a.for_each(|a| { *b_ref = b_ref.push_back(a * 2); });
        assert_eq(b, &[2, 4, 6]);
    }

    #[test]
    fn for_eachi_example() {
        let a = &[1, 2, 3];
        let mut b = &[];
        let b_ref = &mut b;
        a.for_eachi(|i, a| { *b_ref = b_ref.push_back(i + a * 2); });
        assert_eq(b, &[2, 5, 8]);
    }

    #[test]
    fn len_empty() {
        let empty: [Field] = &[];
        assert_eq(empty.len(), 0);
    }

    #[test]
    fn len_single() {
        assert_eq(&[42].len(), 1);
    }

    #[test]
    fn len_multiple() {
        assert_eq(&[1, 2, 3, 4, 5].len(), 5);
    }

    #[test]
    fn push_back_empty() {
        let empty: [Field] = &[];
        let result = empty.push_back(42);
        assert_eq(result.len(), 1);
        assert_eq(result[0], 42);
    }

    #[test]
    fn push_back_non_empty() {
        let slice = &[1, 2, 3];
        let result = slice.push_back(4);
        assert_eq(result.len(), 4);
        assert_eq(result, &[1, 2, 3, 4]);
    }

    #[test]
    fn push_front_empty() {
        let empty = &[];
        let result = empty.push_front(42);
        assert_eq(result.len(), 1);
        assert_eq(result[0], 42);
    }

    #[test]
    fn push_front_non_empty() {
        let slice = &[1, 2, 3];
        let result = slice.push_front(0);
        assert_eq(result.len(), 4);
        assert_eq(result, &[0, 1, 2, 3]);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn pop_back_empty() {
        let slice: [Field] = &[];
        let (_, _) = slice.pop_back();
    }

    #[test]
    fn pop_back_one() {
        let slice = &[42];
        let (result, elem) = slice.pop_back();
        assert_eq(result.len(), 0);
        assert_eq(elem, 42);
    }

    #[test]
    fn pop_back_multiple() {
        let slice = &[1, 2, 3];
        let (result, elem) = slice.pop_back();
        assert_eq(result.len(), 2);
        assert_eq(result, &[1, 2]);
        assert_eq(elem, 3);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn pop_front_empty() {
        let slice: [Field] = &[];
        let (_, _) = slice.pop_front();
    }

    #[test]
    fn pop_front_one() {
        let slice = &[42];
        let (elem, result) = slice.pop_front();
        assert_eq(result.len(), 0);
        assert_eq(elem, 42);
    }

    #[test]
    fn pop_front_multiple() {
        let slice = &[1, 2, 3];
        let (elem, result) = slice.pop_front();
        assert_eq(result.len(), 2);
        assert_eq(result, &[2, 3]);
        assert_eq(elem, 1);
    }

    #[test]
    fn insert_beginning() {
        let slice = &[1, 2, 3];
        let result = slice.insert(0, 0);
        assert_eq(result.len(), 4);
        assert_eq(result, &[0, 1, 2, 3]);
    }

    #[test]
    fn insert_middle() {
        let slice = &[1, 2, 3];
        let result = slice.insert(1, 99);
        assert_eq(result.len(), 4);
        assert_eq(result, &[1, 99, 2, 3]);
    }

    #[test]
    fn insert_end() {
        let slice = &[1, 2, 3];
        let result = slice.insert(3, 4);
        assert_eq(result.len(), 4);
        assert_eq(result, &[1, 2, 3, 4]);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn insert_end_out_of_bounds() {
        let slice = &[1, 2];
        let _ = slice.insert(3, 4);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn remove_empty() {
        let slice: [Field] = &[];
        let (_, _) = slice.remove(0);
    }

    #[test]
    fn remove_beginning() {
        let slice = &[1, 2, 3];
        let (result, elem) = slice.remove(0);
        assert_eq(result.len(), 2);
        assert_eq(result, &[2, 3]);
        assert_eq(elem, 1);
    }

    #[test]
    fn remove_middle() {
        let slice = &[1, 2, 3];
        let (result, elem) = slice.remove(1);
        assert_eq(result.len(), 2);
        assert_eq(result, &[1, 3]);
        assert_eq(elem, 2);
    }

    #[test]
    fn remove_end() {
        let slice = &[1, 2, 3];
        let (result, elem) = slice.remove(2);
        assert_eq(result.len(), 2);
        assert_eq(result, &[1, 2]);
        assert_eq(elem, 3);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn remove_end_out_of_bounds() {
        let slice = &[1, 2];
        let (_, _) = slice.remove(2);
    }

    #[test]
    fn fold_empty() {
        let empty = &[];
        let result = empty.fold(10, |acc, x| acc + x);
        assert_eq(result, 10);
    }

    #[test]
    fn fold_single() {
        let slice = &[5];
        let result = slice.fold(10, |acc, x| acc + x);
        assert_eq(result, 15);
    }

    #[test]
    fn fold_multiple() {
        let slice = &[1, 2, 3, 4];
        let result = slice.fold(0, |acc, x| acc + x);
        assert_eq(result, 10);
    }

    #[test(should_fail_with = "Index out of bounds")]
    fn reduce_empty() {
        let empty: [Field] = &[];
        let _ = empty.reduce(|a, b| a + b);
    }

    #[test]
    fn reduce_single() {
        let slice = &[42];
        let result = slice.reduce(|a, b| a + b);
        assert_eq(result, 42);
    }

    #[test]
    fn reduce_multiple() {
        let slice = &[1, 2, 3, 4];
        let result = slice.reduce(|a, b| a + b);
        assert_eq(result, 10);
    }

    #[test]
    fn filter_empty() {
        let empty = &[];
        let result = empty.filter(|x| x > 0);
        assert_eq(result.len(), 0);
    }

    #[test]
    fn filter_all_true() {
        let slice = &[1, 2, 3, 4];
        let result = slice.filter(|x| x > 0);
        assert_eq(result, slice);
    }

    #[test]
    fn filter_all_false() {
        let slice = &[1, 2, 3, 4];
        let result = slice.filter(|x| x > 10);
        assert_eq(result.len(), 0);
    }

    #[test]
    fn filter_some() {
        let slice = &[1, 2, 3, 4, 5];
        let result = slice.filter(|x| x % 2 == 0);
        assert_eq(result, &[2, 4]);
    }

    #[test]
    fn all_empty() {
        let empty = &[];
        let result = empty.all(|x| x > 0);
        assert_eq(result, true);
    }

    #[test]
    fn all_true() {
        let slice = &[1, 2, 3, 4];
        let result = slice.all(|x| x > 0);
        assert_eq(result, true);
    }

    #[test]
    fn all_false() {
        let slice = &[1, 2, 3, 4];
        let result = slice.all(|x| x > 2);
        assert_eq(result, false);
    }

    #[test]
    fn any_empty() {
        let empty = &[];
        let result = empty.any(|x| x > 0);
        assert_eq(result, false);
    }

    #[test]
    fn any_true() {
        let slice = &[1, 2, 3, 4];
        let result = slice.any(|x| x > 3);
        assert_eq(result, true);
    }

    #[test]
    fn any_false() {
        let slice = &[1, 2, 3, 4];
        let result = slice.any(|x| x > 10);
        assert_eq(result, false);
    }

    // utility method tests
    #[test]
    fn append_empty_to_empty() {
        let empty1: [Field] = &[];
        let empty2: [Field] = &[];
        let result = empty1.append(empty2);
        assert_eq(result.len(), 0);
    }

    #[test]
    fn append_empty_to_non_empty() {
        let slice = &[1, 2, 3];
        let empty = &[];
        let result = slice.append(empty);
        assert_eq(result, slice);
    }

    #[test]
    fn append_non_empty_to_empty() {
        let empty = &[];
        let slice = &[1, 2, 3];
        let result = empty.append(slice);
        assert_eq(result, slice);
    }

    #[test]
    fn append_two_non_empty() {
        let slice1 = &[1, 2];
        let slice2 = &[3, 4, 5];
        let result = slice1.append(slice2);
        assert_eq(result, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn as_array_single() {
        let slice = &[42];
        let array: [Field; 1] = slice.as_array();
        assert_eq(array[0], 42);
    }

    #[test]
    fn as_array_multiple() {
        let slice = &[1, 2, 3];
        let array: [Field; 3] = slice.as_array();
        assert_eq(array[0], 1);
        assert_eq(array[1], 2);
        assert_eq(array[2], 3);
    }

    // complex scenarios
    #[test]
    fn chain_operations() {
        let slice = &[1, 2, 3, 4, 5];
        let result = slice.filter(|x| x % 2 == 0).map(|x| x * 2).fold(0, |acc, x| acc + x);
        assert_eq(result, 12); // (2*2) + (4*2) = 4 + 8 = 12
    }

    #[test]
    fn nested_operations() {
        let slice = &[1, 2, 3, 4];
        let filtered = slice.filter(|x| x > 1);
        let mapped = filtered.map(|x| x * x);
        let sum = mapped.fold(0, |acc, x| acc + x);
        assert_eq(sum, 29); // 2^2 + 3^2 + 4^2 = 4 + 9 + 16 = 29
    }

    #[test]
    fn single_element_operations() {
        let single = &[42];

        // Test all operations on single element
        assert_eq(single.len(), 1);

        let pushed_back = single.push_back(99);
        assert_eq(pushed_back, &[42, 99]);

        let pushed_front = single.push_front(0);
        assert_eq(pushed_front, &[0, 42]);

        let (popped_back_slice, popped_back_elem) = single.pop_back();
        assert_eq(popped_back_slice.len(), 0);
        assert_eq(popped_back_elem, 42);

        let (popped_front_elem, popped_front_slice) = single.pop_front();
        assert_eq(popped_front_slice.len(), 0);
        assert_eq(popped_front_elem, 42);

        let inserted = single.insert(0, 0);
        assert_eq(inserted, &[0, 42]);

        let (removed_slice, removed_elem) = single.remove(0);
        assert_eq(removed_slice.len(), 0);
        assert_eq(removed_elem, 42);
    }

    #[test]
    fn boundary_conditions() {
        let slice = &[1, 2, 3];

        // insert at boundaries
        let at_start = slice.insert(0, 0);
        assert_eq(at_start, &[0, 1, 2, 3]);

        let at_end = slice.insert(3, 4);
        assert_eq(at_end, &[1, 2, 3, 4]);

        // remove at boundaries
        let (removed_start, elem_start) = slice.remove(0);
        assert_eq(removed_start, &[2, 3]);
        assert_eq(elem_start, 1);

        let (removed_end, elem_end) = slice.remove(2);
        assert_eq(removed_end, &[1, 2]);
        assert_eq(elem_end, 3);
    }

    #[test]
    fn complex_predicates() {
        let slice = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let even_greater_than_5 = slice.filter(|x| (x % 2 == 0) & (x > 5));
        assert_eq(even_greater_than_5, &[6, 8, 10]);

        let all_positive_and_less_than_20 = slice.all(|x| (x > 0) & (x < 20));
        assert_eq(all_positive_and_less_than_20, true);

        let any_divisible_by_7 = slice.any(|x| x % 7 == 0);
        assert_eq(any_divisible_by_7, true);
    }

    #[test]
    fn identity_operations() {
        let slice = &[1, 2, 3, 4, 5];

        let mapped_identity = slice.map(|x| x);
        assert_eq(mapped_identity, slice);

        let filtered_all = slice.filter(|_| true);
        assert_eq(filtered_all, slice);

        let filtered_none = slice.filter(|_| false);
        assert_eq(filtered_none.len(), 0);
    }

    #[test(should_fail)]
    fn as_array_size_mismatch() {
        let slice = &[1, 2, 3];
        let _: [Field; 5] = slice.as_array(); // size doesn't match
    }
}
