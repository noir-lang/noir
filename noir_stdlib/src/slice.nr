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

    /// Returns a new slice with elements in reverse order.
    /// The original slice remains untouched.
    ///
    /// Example:
    /// ```noir
    /// fn main() {
    ///     let slice = &[1, 2, 3];
    ///     let reversed = slice.reverse();
    ///     assert(reversed == &[3, 2, 1]);
    /// }
    /// ```
    pub fn reverse(self) -> Self {
        let mut result = &[];
        let len = self.len();
        
        for i in 0..len {
            result = result.push_front(self[len - 1 - i]);
        }
        
        result
    }
}

/// Creates a new slice containing numbers from 0 to size-1.
/// This is a utility function to create slices from ranges.
///
/// Example:
/// ```noir
/// fn main() {
///     let slice = std::slice::range(5);
///     assert(slice == &[0, 1, 2, 3, 4]);
/// }
/// ```
pub fn range(size: u32) -> [u32] {
    let mut result = &[];
    for i in 0..size {
        result = result.push_back(i);
    }
    result
}

/// Creates a new slice by applying a function to each index.
/// This is a utility function to create slices with custom initialization.
///
/// Example:
/// ```noir
/// fn main() {
///     let slice = std::slice::from_fn(5, |i| i * 2);
///     assert(slice == &[0, 2, 4, 6, 8]);
/// }
/// ```
pub fn from_fn<T>(size: u32, f: fn(u32) -> T) -> [T] {
    let mut result = &[];
    for i in 0..size {
        result = result.push_back(f(i));
    }
    result
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
    }

    #[test]
    fn for_eachi_empty() {
        let empty_slice: [Field] = &[];
        empty_slice.for_eachi(|_i, _x| assert(false));
    }

    #[test]
    fn map_example() {
        let a = &[1, 2, 3];
        let b = a.map(|a| a * 2);
        assert_eq(b, &[2, 4, 6]);
    }

    #[test]
    fn mapi_example() {
        let a = &[1, 2, 3];
        let b = a.mapi(|i, a| i * a + 1);
        assert_eq(b, &[1, 3, 7]);
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
    fn test_slice_range() {
        let slice = range(5);
        assert_eq(slice, &[0_u32, 1_u32, 2_u32, 3_u32, 4_u32]);

        let empty = range(0);
        assert_eq(empty, &[]);
    }
    
    #[test]
    fn test_slice_from_fn() {
        let slice = from_fn(5, |i| i * 2);
        assert_eq(slice, &[0_u32, 2_u32, 4_u32, 6_u32, 8_u32]);
        
        let field_slice = from_fn(3, |i| i as Field);
        assert_eq(field_slice, &[0_Field, 1_Field, 2_Field]);
    }
    
    #[test]
    fn test_slice_reverse() {
        let slice = &[1, 2, 3, 4, 5];
        let reversed = slice.reverse();
        assert_eq(reversed, &[5, 4, 3, 2, 1]);
        
        let empty_slice: [Field] = &[];
        assert_eq(empty_slice.reverse(), &[]);
        
        let single = &[1];
        assert_eq(single.reverse(), &[1]);
    }
}
