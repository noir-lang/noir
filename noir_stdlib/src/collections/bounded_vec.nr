use crate::{cmp::Eq, convert::From, runtime::is_unconstrained, static_assert};

/// A `BoundedVec<T, MaxLen>` is a growable storage similar to a `Vec<T>` except that it
/// is bounded with a maximum possible length. Unlike `Vec`, `BoundedVec` is not implemented
/// via slices and thus is not subject to the same restrictions slices are (notably, nested
/// slices - and thus nested vectors as well - are disallowed).
///
/// Since a BoundedVec is backed by a normal array under the hood, growing the BoundedVec by
/// pushing an additional element is also more efficient - the length only needs to be increased
/// by one.
///
/// For these reasons `BoundedVec<T, N>` should generally be preferred over `Vec<T>` when there
/// is a reasonable maximum bound that can be placed on the vector.
///
/// Example:
///
/// ```noir
/// let mut vector: BoundedVec<Field, 10> = BoundedVec::new();
/// for i in 0..5 {
///     vector.push(i);
/// }
/// assert(vector.len() == 5);
/// assert(vector.max_len() == 10);
/// ```
pub struct BoundedVec<T, let MaxLen: u32> {
    storage: [T; MaxLen],
    len: u32,
}

impl<T, let MaxLen: u32> BoundedVec<T, MaxLen> {
    /// Creates a new, empty vector of length zero.
    ///
    /// Since this container is backed by an array internally, it still needs an initial value
    /// to give each element. To resolve this, each element is zeroed internally. This value
    /// is guaranteed to be inaccessible unless `get_unchecked` is used.
    ///
    /// Example:
    ///
    /// ```noir
    /// let empty_vector: BoundedVec<Field, 10> = BoundedVec::new();
    /// assert(empty_vector.len() == 0);
    /// ```
    ///
    /// Note that whenever calling `new` the maximum length of the vector should always be specified
    /// via a type signature:
    ///
    /// ```noir
    /// fn good() -> BoundedVec<Field, 10> {
    ///     // Ok! MaxLen is specified with a type annotation
    ///     let v1: BoundedVec<Field, 3> = BoundedVec::new();
    ///     let v2 = BoundedVec::new();
    ///
    ///     // Ok! MaxLen is known from the type of `good`'s return value
    ///     v2
    /// }
    ///
    /// fn bad() {
    ///     // Error: Type annotation needed
    ///     // The compiler can't infer `MaxLen` from the following code:
    ///     let mut v3 = BoundedVec::new();
    ///     v3.push(5);
    /// }
    /// ```
    ///
    /// This defaulting of `MaxLen` (and numeric generics in general) to zero may change in future noir versions
    /// but for now make sure to use type annotations when using bounded vectors. Otherwise, you will receive a
    /// constraint failure at runtime when the vec is pushed to.
    pub fn new() -> Self {
        let zeroed = crate::mem::zeroed();
        BoundedVec { storage: [zeroed; MaxLen], len: 0 }
    }

    /// Retrieves an element from the vector at the given index, starting from zero.
    ///
    /// If the given index is equal to or greater than the length of the vector, this
    /// will issue a constraint failure.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn foo<let N: u32>(v: BoundedVec<u32, N>) {
    ///     let first = v.get(0);
    ///     let last = v.get(v.len() - 1);
    ///     assert(first != last);
    /// }
    /// ```
    pub fn get(self, index: u32) -> T {
        assert(index < self.len, "Attempted to read past end of BoundedVec");
        self.get_unchecked(index)
    }

    /// Retrieves an element from the vector at the given index, starting from zero, without
    /// performing a bounds check.
    ///
    /// Since this function does not perform a bounds check on length before accessing the element,
    /// it is unsafe! Use at your own risk!
    ///
    /// Example:
    ///
    /// ```noir
    /// fn sum_of_first_three<let N: u32>(v: BoundedVec<u32, N>) -> u32 {
    ///     // Always ensure the length is larger than the largest
    ///     // index passed to get_unchecked
    ///     assert(v.len() > 2);
    ///     let first = v.get_unchecked(0);
    ///     let second = v.get_unchecked(1);
    ///     let third = v.get_unchecked(2);
    ///     first + second + third
    /// }
    /// ```
    pub fn get_unchecked(self, index: u32) -> T {
        self.storage[index]
    }

    /// Writes an element to the vector at the given index, starting from zero.
    ///
    /// If the given index is equal to or greater than the length of the vector, this will issue a constraint failure.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn foo<let N: u32>(v: BoundedVec<u32, N>) {
    ///     let first = v.get(0);
    ///     assert(first != 42);
    ///     v.set(0, 42);
    ///     let new_first = v.get(0);
    ///     assert(new_first == 42);
    /// }
    /// ```
    pub fn set(&mut self, index: u32, value: T) {
        assert(index < self.len, "Attempted to write past end of BoundedVec");
        self.set_unchecked(index, value)
    }

    /// Writes an element to the vector at the given index, starting from zero, without performing a bounds check.
    ///
    /// Since this function does not perform a bounds check on length before accessing the element, it is unsafe! Use at your own risk!
    ///
    /// Example:
    ///
    /// ```noir
    /// fn set_unchecked_example() {
    ///     let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
    ///     vec.extend_from_array([1, 2]);
    ///
    ///     // Here we're safely writing within the valid range of `vec`
    ///     // `vec` now has the value [42, 2]
    ///     vec.set_unchecked(0, 42);
    ///
    ///     // We can then safely read this value back out of `vec`.
    ///     // Notice that we use the checked version of `get` which would prevent reading unsafe values.
    ///     assert_eq(vec.get(0), 42);
    ///
    ///     // We've now written past the end of `vec`.
    ///     // As this index is still within the maximum potential length of `v`,
    ///     // it won't cause a constraint failure.
    ///     vec.set_unchecked(2, 42);
    ///     println(vec);
    ///
    ///     // This will write past the end of the maximum potential length of `vec`,
    ///     // it will then trigger a constraint failure.
    ///     vec.set_unchecked(5, 42);
    ///     println(vec);
    /// }
    /// ```
    pub fn set_unchecked(&mut self, index: u32, value: T) {
        self.storage[index] = value;
    }

    /// Pushes an element to the end of the vector. This increases the length
    /// of the vector by one.
    ///
    /// Panics if the new length of the vector will be greater than the max length.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<Field, 2> = BoundedVec::new();
    ///
    /// v.push(1);
    /// v.push(2);
    ///
    /// // Panics with failed assertion "push out of bounds"
    /// v.push(3);
    /// ```
    pub fn push(&mut self, elem: T) {
        assert(self.len < MaxLen, "push out of bounds");

        self.storage[self.len] = elem;
        self.len += 1;
    }

    /// Returns the current length of this vector
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<Field, 4> = BoundedVec::new();
    /// assert(v.len() == 0);
    ///
    /// v.push(100);
    /// assert(v.len() == 1);
    ///
    /// v.push(200);
    /// v.push(300);
    /// v.push(400);
    /// assert(v.len() == 4);
    ///
    /// let _ = v.pop();
    /// let _ = v.pop();
    /// assert(v.len() == 2);
    /// ```
    pub fn len(self) -> u32 {
        self.len
    }

    /// Returns the maximum length of this vector. This is always
    /// equal to the `MaxLen` parameter this vector was initialized with.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<Field, 5> = BoundedVec::new();
    ///
    /// assert(v.max_len() == 5);
    /// v.push(10);
    /// assert(v.max_len() == 5);
    /// ```
    pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u32 {
        MaxLen
    }

    /// Returns the internal array within this vector.
    ///
    /// Since arrays in Noir are immutable, mutating the returned storage array will not mutate
    /// the storage held internally by this vector.
    ///
    /// Note that uninitialized elements may be zeroed out!
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<Field, 5> = BoundedVec::new();
    ///
    /// assert(v.storage() == [0, 0, 0, 0, 0]);
    ///
    /// v.push(57);
    /// assert(v.storage() == [57, 0, 0, 0, 0]);
    /// ```
    pub fn storage(self) -> [T; MaxLen] {
        self.storage
    }

    /// Pushes each element from the given array to this vector.
    ///
    /// Panics if pushing each element would cause the length of this vector
    /// to exceed the maximum length.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    /// vec.extend_from_array([2, 4]);
    ///
    /// assert(vec.len == 2);
    /// assert(vec.get(0) == 2);
    /// assert(vec.get(1) == 4);
    /// ```
    pub fn extend_from_array<let Len: u32>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();
        assert(new_len <= MaxLen, "extend_from_array out of bounds");
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
    }

    /// Pushes each element from the given slice to this vector.
    ///
    /// Panics if pushing each element would cause the length of this vector
    /// to exceed the maximum length.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    /// vec.extend_from_slice(&[2, 4]);
    ///
    /// assert(vec.len == 2);
    /// assert(vec.get(0) == 2);
    /// assert(vec.get(1) == 4);
    /// ```
    pub fn extend_from_slice(&mut self, slice: [T]) {
        let new_len = self.len + slice.len();
        assert(new_len <= MaxLen, "extend_from_slice out of bounds");
        for i in 0..slice.len() {
            self.storage[self.len + i] = slice[i];
        }
        self.len = new_len;
    }

    /// Pushes each element from the other vector to this vector. The length of
    /// the other vector is left unchanged.
    ///
    /// Panics if pushing each element would cause the length of this vector
    /// to exceed the maximum length.
    ///
    /// ```noir
    /// let mut v1: BoundedVec<Field, 5> = BoundedVec::new();
    /// let mut v2: BoundedVec<Field, 7> = BoundedVec::new();
    ///
    /// v2.extend_from_array([1, 2, 3]);
    /// v1.extend_from_bounded_vec(v2);
    ///
    /// assert(v1.storage() == [1, 2, 3, 0, 0]);
    /// assert(v2.storage() == [1, 2, 3, 0, 0, 0, 0]);
    /// ```
    pub fn extend_from_bounded_vec<let Len: u32>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len <= MaxLen, "extend_from_bounded_vec out of bounds");

        if is_unconstrained() {
            for i in 0..append_len {
                self.storage[self.len + i] = vec.get_unchecked(i);
            }
        } else {
            let mut exceeded_len = false;
            for i in 0..Len {
                exceeded_len |= i == append_len;
                if !exceeded_len {
                    self.storage[self.len + i] = vec.get_unchecked(i);
                }
            }
        }
        self.len = new_len;
    }

    /// Creates a new vector, populating it with values derived from an array input.
    /// The maximum length of the vector is determined based on the type signature.
    ///
    /// Example:
    ///
    /// ```noir
    /// let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from_array([1, 2, 3])
    /// ```
    pub fn from_array<let Len: u32>(array: [T; Len]) -> Self {
        static_assert(Len <= MaxLen, "from array out of bounds");
        let mut vec: BoundedVec<T, MaxLen> = BoundedVec::new();
        vec.extend_from_array(array);
        vec
    }

    /// Pops the element at the end of the vector. This will decrease the length
    /// of the vector by one.
    ///
    /// Panics if the vector is empty.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<Field, 2> = BoundedVec::new();
    /// v.push(1);
    /// v.push(2);
    ///
    /// let two = v.pop();
    /// let one = v.pop();
    ///
    /// assert(two == 2);
    /// assert(one == 1);
    ///
    /// // error: cannot pop from an empty vector
    /// let _ = v.pop();
    /// ```
    pub fn pop(&mut self) -> T {
        assert(self.len > 0, "cannot pop from an empty vector");
        self.len -= 1;

        let elem = self.storage[self.len];
        self.storage[self.len] = crate::mem::zeroed();
        elem
    }

    /// Returns true if the given predicate returns true for any element
    /// in this vector.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: BoundedVec<u32, 3> = BoundedVec::new();
    /// v.extend_from_array([2, 4, 6]);
    ///
    /// let all_even = !v.any(|elem: u32| elem % 2 != 0);
    /// assert(all_even);
    /// ```
    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        if is_unconstrained() {
            for i in 0..self.len {
                ret |= predicate(self.storage[i]);
            }
        } else {
            let mut exceeded_len = false;
            for i in 0..MaxLen {
                exceeded_len |= i == self.len;
                if !exceeded_len {
                    ret |= predicate(self.storage[i]);
                }
            }
        }
        ret
    }

    /// Creates a new vector of equal size by calling a closure on each element in this vector.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
    /// let result = vec.map(|value| value * 2);
    ///
    /// let expected = BoundedVec::from_array([2, 4, 6, 8]);
    /// assert_eq(result, expected);
    /// ```
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> BoundedVec<U, MaxLen> {
        let mut ret = BoundedVec::new();
        ret.len = self.len();

        if is_unconstrained() {
            for i in 0..self.len() {
                ret.storage[i] = f(self.get_unchecked(i));
            }
        } else {
            for i in 0..MaxLen {
                if i < self.len() {
                    ret.storage[i] = f(self.get_unchecked(i));
                }
            }
        }

        ret
    }

    /// Creates a new vector of equal size by calling a closure on each element
    /// in this vector, along with its index.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
    /// let result = vec.mapi(|i, value| i + value * 2);
    ///
    /// let expected = BoundedVec::from_array([2, 5, 8, 11]);
    /// assert_eq(result, expected);
    /// ```
    pub fn mapi<U, Env>(self, f: fn[Env](u32, T) -> U) -> BoundedVec<U, MaxLen> {
        let mut ret = BoundedVec::new();
        ret.len = self.len();

        if is_unconstrained() {
            for i in 0..self.len() {
                ret.storage[i] = f(i, self.get_unchecked(i));
            }
        } else {
            for i in 0..MaxLen {
                if i < self.len() {
                    ret.storage[i] = f(i, self.get_unchecked(i));
                }
            }
        }

        ret
    }

    /// Calls a closure on each element in this vector.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
    /// let mut result = BoundedVec::<u32, 4>::new();
    /// vec.for_each(|value| result.push(value * 2));
    ///
    /// let expected = BoundedVec::from_array([2, 4, 6, 8]);
    /// assert_eq(result, expected);
    /// ```
    pub fn for_each<Env>(self, f: fn[Env](T) -> ()) {
        if is_unconstrained() {
            for i in 0..self.len() {
                f(self.get_unchecked(i));
            }
        } else {
            for i in 0..MaxLen {
                if i < self.len() {
                    f(self.get_unchecked(i));
                }
            }
        }
    }

    /// Calls a closure on each element in this vector, along with its index.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
    /// let mut result = BoundedVec::<u32, 4>::new();
    /// vec.for_eachi(|i, value| result.push(i + value * 2));
    ///
    /// let expected = BoundedVec::from_array([2, 5, 8, 11]);
    /// assert_eq(result, expected);
    /// ```
    pub fn for_eachi<Env>(self, f: fn[Env](u32, T) -> ()) {
        if is_unconstrained() {
            for i in 0..self.len() {
                f(i, self.get_unchecked(i));
            }
        } else {
            for i in 0..MaxLen {
                if i < self.len() {
                    f(i, self.get_unchecked(i));
                }
            }
        }
    }

    /// Creates a new BoundedVec from the given array and length.
    /// The given length must be less than or equal to the length of the array.
    ///
    /// This function will zero out any elements at or past index `len` of `array`.
    /// This incurs an extra runtime cost of O(MaxLen). If you are sure your array is
    /// zeroed after that index, you can use `from_parts_unchecked` to remove the extra loop.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 0], 3);
    /// assert_eq(vec.len(), 3);
    /// ```
    pub fn from_parts(mut array: [T; MaxLen], len: u32) -> Self {
        assert(len <= MaxLen);
        let zeroed = crate::mem::zeroed();

        if is_unconstrained() {
            for i in len..MaxLen {
                array[i] = zeroed;
            }
        } else {
            for i in 0..MaxLen {
                if i >= len {
                    array[i] = zeroed;
                }
            }
        }

        BoundedVec { storage: array, len }
    }

    /// Creates a new BoundedVec from the given array and length.
    /// The given length must be less than or equal to the length of the array.
    ///
    /// This function is unsafe because it expects all elements past the `len` index
    /// of `array` to be zeroed, but does not check for this internally. Use `from_parts`
    /// for a safe version of this function which does zero out any indices past the
    /// given length. Invalidating this assumption can notably cause `BoundedVec::eq`
    /// to give incorrect results since it will check even elements past `len`.
    ///
    /// Example:
    ///
    /// ```noir
    /// let vec: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 0], 3);
    /// assert_eq(vec.len(), 3);
    ///
    /// // invalid use!
    /// let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 1], 3);
    /// let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 2], 3);
    ///
    /// // both vecs have length 3 so we'd expect them to be equal, but this
    /// // fails because elements past the length are still checked in eq
    /// assert_eq(vec1, vec2); // fails
    /// ```
    pub fn from_parts_unchecked(array: [T; MaxLen], len: u32) -> Self {
        assert(len <= MaxLen);
        BoundedVec { storage: array, len }
    }
}

impl<T, let MaxLen: u32> Eq for BoundedVec<T, MaxLen>
where
    T: Eq,
{
    fn eq(self, other: BoundedVec<T, MaxLen>) -> bool {
        // TODO: https://github.com/noir-lang/noir/issues/4837
        //
        // We make the assumption that the user has used the proper interface for working with `BoundedVec`s
        // rather than directly manipulating the internal fields as this can result in an inconsistent internal state.
        if self.len == other.len {
            self.storage == other.storage
        } else {
            false
        }
    }
}

impl<T, let MaxLen: u32, let Len: u32> From<[T; Len]> for BoundedVec<T, MaxLen> {
    fn from(array: [T; Len]) -> BoundedVec<T, MaxLen> {
        BoundedVec::from_array(array)
    }
}

mod bounded_vec_tests {

    mod get {
        use crate::collections::bounded_vec::BoundedVec;

        #[test(should_fail_with = "Attempted to read past end of BoundedVec")]
        fn panics_when_reading_elements_past_end_of_vec() {
            let vec: BoundedVec<Field, 5> = BoundedVec::new();

            let _ = vec.get(0);
        }

        #[test(should_fail_with = "Attempted to read past end of BoundedVec")]
        fn panics_when_reading_beyond_length() {
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3]);
            let _ = vec.get(3);
        }

        #[test]
        fn get_works_within_bounds() {
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3, 4, 5]);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(4), 5);
        }

        #[test]
        fn get_unchecked_works() {
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3]);
            assert_eq(vec.get_unchecked(0), 1);
            assert_eq(vec.get_unchecked(2), 3);
        }

        #[test]
        fn get_unchecked_works_past_len() {
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3]);
            assert_eq(vec.get_unchecked(4), 0);
        }
    }

    mod set {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn set_updates_values_properly() {
            let mut vec = BoundedVec::from_array([0, 0, 0, 0, 0]);

            vec.set(0, 42);
            assert_eq(vec.storage, [42, 0, 0, 0, 0]);

            vec.set(1, 43);
            assert_eq(vec.storage, [42, 43, 0, 0, 0]);

            vec.set(2, 44);
            assert_eq(vec.storage, [42, 43, 44, 0, 0]);

            vec.set(1, 10);
            assert_eq(vec.storage, [42, 10, 44, 0, 0]);

            vec.set(0, 0);
            assert_eq(vec.storage, [0, 10, 44, 0, 0]);
        }

        #[test(should_fail_with = "Attempted to write past end of BoundedVec")]
        fn panics_when_writing_elements_past_end_of_vec() {
            let mut vec: BoundedVec<Field, 5> = BoundedVec::new();
            vec.set(0, 42);
        }

        #[test(should_fail_with = "Attempted to write past end of BoundedVec")]
        fn panics_when_setting_beyond_length() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3]);
            vec.set(3, 4);
        }

        #[test]
        fn set_unchecked_operations() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            vec.push(1);
            vec.push(2);

            vec.set_unchecked(0, 10);
            assert_eq(vec.get(0), 10);
        }

        #[test(should_fail_with = "Attempted to read past end of BoundedVec")]
        fn set_unchecked_operations_past_len() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            vec.push(1);
            vec.push(2);

            vec.set_unchecked(3, 40);
            assert_eq(vec.get(3), 40);
        }

        #[test]
        fn set_preserves_other_elements() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3, 4, 5]);

            vec.set(2, 30);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 30);
            assert_eq(vec.get(3), 4);
            assert_eq(vec.get(4), 5);
        }
    }

    mod any {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn returns_false_if_predicate_not_satisfied() {
            let vec: BoundedVec<bool, 4> = BoundedVec::from_array([false, false, false, false]);
            let result = vec.any(|value| value);

            assert(!result);
        }

        #[test]
        fn returns_true_if_predicate_satisfied() {
            let vec: BoundedVec<bool, 4> = BoundedVec::from_array([false, false, true, true]);
            let result = vec.any(|value| value);

            assert(result);
        }

        #[test]
        fn returns_false_on_empty_boundedvec() {
            let vec: BoundedVec<bool, 0> = BoundedVec::new();
            let result = vec.any(|value| value);

            assert(!result);
        }

        #[test]
        fn any_with_complex_predicates() {
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3, 4, 5]);

            assert(vec.any(|x| x > 3));
            assert(!vec.any(|x| x > 10));
            assert(vec.any(|x| x % 2 == 0)); // has a even number
            assert(vec.any(|x| x == 3)); // has a specific value
        }

        #[test]
        fn any_with_partial_vector() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            vec.push(1);
            vec.push(2);

            assert(vec.any(|x| x == 1));
            assert(vec.any(|x| x == 2));
            assert(!vec.any(|x| x == 3));
        }
    }

    mod map {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn applies_function_correctly() {
            // docs:start:bounded-vec-map-example
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.map(|value| value * 2);
            // docs:end:bounded-vec-map-example
            let expected = BoundedVec::from_array([2, 4, 6, 8]);

            assert_eq(result, expected);
        }

        #[test]
        fn applies_function_that_changes_return_type() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.map(|value| (value * 2) as Field);
            let expected: BoundedVec<Field, 4> = BoundedVec::from_array([2, 4, 6, 8]);

            assert_eq(result, expected);
        }

        #[test]
        fn does_not_apply_function_past_len() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([0, 1]);
            let result = vec.map(|value| if value == 0 { 5 } else { value });
            let expected = BoundedVec::from_array([5, 1]);

            assert_eq(result, expected);
            assert_eq(result.get_unchecked(2), 0);
        }

        #[test]
        fn map_with_conditional_logic() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);

            let result = vec.map(|x| if x % 2 == 0 { x * 2 } else { x });
            let expected = BoundedVec::from_array([1, 4, 3, 8]);
            assert_eq(result, expected);
        }

        #[test]
        fn map_preserves_length() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.map(|x| x * 2);

            assert_eq(result.len(), vec.len());
            assert_eq(result.max_len(), vec.max_len());
        }

        #[test]
        fn map_on_empty_vector() {
            let vec: BoundedVec<u32, 5> = BoundedVec::new();
            let result = vec.map(|x| x * 2);
            assert_eq(result, vec);
            assert_eq(result.len(), 0);
            assert_eq(result.max_len(), 5);
        }
    }

    mod mapi {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn applies_function_correctly() {
            // docs:start:bounded-vec-mapi-example
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.mapi(|i, value| i + value * 2);
            // docs:end:bounded-vec-mapi-example
            let expected = BoundedVec::from_array([2, 5, 8, 11]);

            assert_eq(result, expected);
        }

        #[test]
        fn applies_function_that_changes_return_type() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.mapi(|i, value| (i + value * 2) as Field);
            let expected: BoundedVec<Field, 4> = BoundedVec::from_array([2, 5, 8, 11]);

            assert_eq(result, expected);
        }

        #[test]
        fn does_not_apply_function_past_len() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([0, 1]);
            let result = vec.mapi(|_, value| if value == 0 { 5 } else { value });
            let expected = BoundedVec::from_array([5, 1]);

            assert_eq(result, expected);
            assert_eq(result.get_unchecked(2), 0);
        }

        #[test]
        fn mapi_with_index_branching_logic() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);

            let result = vec.mapi(|i, x| if i % 2 == 0 { x * 2 } else { x });
            let expected = BoundedVec::from_array([2, 2, 6, 4]);
            assert_eq(result, expected);
        }
    }

    mod for_each {
        use crate::collections::bounded_vec::BoundedVec;

        // map in terms of for_each
        fn for_each_map<T, U, Env, let MaxLen: u32>(
            input: BoundedVec<T, MaxLen>,
            f: fn[Env](T) -> U,
        ) -> BoundedVec<U, MaxLen> {
            let mut output = BoundedVec::<U, MaxLen>::new();
            let output_ref = &mut output;
            input.for_each(|x| output_ref.push(f(x)));
            output
        }

        #[test]
        fn smoke_test() {
            let mut acc = 0;
            let acc_ref = &mut acc;
            // docs:start:bounded-vec-for-each-example
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([1, 2, 3]);
            vec.for_each(|value| { *acc_ref += value; });
            // docs:end:bounded-vec-for-each-example
            assert_eq(acc, 6);
        }

        #[test]
        fn applies_function_correctly() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = for_each_map(vec, |value| value * 2);
            let expected = BoundedVec::from_array([2, 4, 6, 8]);

            assert_eq(result, expected);
        }

        #[test]
        fn applies_function_that_changes_return_type() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = for_each_map(vec, |value| (value * 2) as Field);
            let expected: BoundedVec<Field, 4> = BoundedVec::from_array([2, 4, 6, 8]);

            assert_eq(result, expected);
        }

        #[test]
        fn does_not_apply_function_past_len() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([0, 1]);
            let result = for_each_map(vec, |value| if value == 0 { 5 } else { value });
            let expected = BoundedVec::from_array([5, 1]);

            assert_eq(result, expected);
            assert_eq(result.get_unchecked(2), 0);
        }

        #[test]
        fn for_each_on_empty_vector() {
            let vec: BoundedVec<u32, 5> = BoundedVec::new();
            let mut count = 0;
            let count_ref = &mut count;
            vec.for_each(|_| { *count_ref += 1; });
            assert_eq(count, 0);
        }

        #[test]
        fn for_each_with_side_effects() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([1, 2, 3]);
            let mut seen = BoundedVec::<u32, 3>::new();
            let seen_ref = &mut seen;
            vec.for_each(|x| seen_ref.push(x));
            assert_eq(seen, vec);
        }
    }

    mod for_eachi {
        use crate::collections::bounded_vec::BoundedVec;

        // mapi in terms of for_eachi
        fn for_eachi_mapi<T, U, Env, let MaxLen: u32>(
            input: BoundedVec<T, MaxLen>,
            f: fn[Env](u32, T) -> U,
        ) -> BoundedVec<U, MaxLen> {
            let mut output = BoundedVec::<U, MaxLen>::new();
            let output_ref = &mut output;
            input.for_eachi(|i, x| output_ref.push(f(i, x)));
            output
        }

        #[test]
        fn smoke_test() {
            let mut acc = 0;
            let acc_ref = &mut acc;
            // docs:start:bounded-vec-for-eachi-example
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([1, 2, 3]);
            vec.for_eachi(|i, value| { *acc_ref += i * value; });
            // docs:end:bounded-vec-for-eachi-example

            // 0 * 1 + 1 * 2 + 2 * 3
            assert_eq(acc, 8);
        }

        #[test]
        fn applies_function_correctly() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = for_eachi_mapi(vec, |i, value| i + value * 2);
            let expected = BoundedVec::from_array([2, 5, 8, 11]);

            assert_eq(result, expected);
        }

        #[test]
        fn applies_function_that_changes_return_type() {
            let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = for_eachi_mapi(vec, |i, value| (i + value * 2) as Field);
            let expected: BoundedVec<Field, 4> = BoundedVec::from_array([2, 5, 8, 11]);

            assert_eq(result, expected);
        }

        #[test]
        fn does_not_apply_function_past_len() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([0, 1]);
            let result = for_eachi_mapi(vec, |_, value| if value == 0 { 5 } else { value });
            let expected = BoundedVec::from_array([5, 1]);

            assert_eq(result, expected);
            assert_eq(result.get_unchecked(2), 0);
        }

        #[test]
        fn for_eachi_on_empty_vector() {
            let vec: BoundedVec<u32, 5> = BoundedVec::new();
            let mut count = 0;
            let count_ref = &mut count;
            vec.for_eachi(|_, _| { *count_ref += 1; });
            assert_eq(count, 0);
        }

        #[test]
        fn for_eachi_with_index_tracking() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([10, 20, 30]);
            let mut indices = BoundedVec::<u32, 3>::new();
            let indices_ref = &mut indices;
            vec.for_eachi(|i, _| indices_ref.push(i));

            let expected = BoundedVec::from_array([0, 1, 2]);
            assert_eq(indices, expected);
        }

    }

    mod from_array {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn empty() {
            let empty_array: [Field; 0] = [];
            let bounded_vec = BoundedVec::from_array([]);

            assert_eq(bounded_vec.max_len(), 0);
            assert_eq(bounded_vec.len(), 0);
            assert_eq(bounded_vec.storage(), empty_array);
        }

        #[test]
        fn equal_len() {
            let array = [1, 2, 3];
            let bounded_vec = BoundedVec::from_array(array);

            assert_eq(bounded_vec.max_len(), 3);
            assert_eq(bounded_vec.len(), 3);
            assert_eq(bounded_vec.storage(), array);
        }

        #[test]
        fn max_len_greater_then_array_len() {
            let array = [1, 2, 3];
            let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from_array(array);

            assert_eq(bounded_vec.max_len(), 10);
            assert_eq(bounded_vec.len(), 3);
            assert_eq(bounded_vec.get(0), 1);
            assert_eq(bounded_vec.get(1), 2);
            assert_eq(bounded_vec.get(2), 3);
        }

        #[test(should_fail_with = "from array out of bounds")]
        fn max_len_lower_then_array_len() {
            let _: BoundedVec<Field, 2> = BoundedVec::from_array([0; 3]);
        }

        #[test]
        fn from_array_preserves_order() {
            let array = [5, 3, 1, 4, 2];
            let vec: BoundedVec<u32, 5> = BoundedVec::from_array(array);
            for i in 0..array.len() {
                assert_eq(vec.get(i), array[i]);
            }
        }

        #[test]
        fn from_array_with_different_types() {
            let bool_array = [true, false, true];
            let bool_vec: BoundedVec<bool, 3> = BoundedVec::from_array(bool_array);
            assert_eq(bool_vec.len(), 3);
            assert_eq(bool_vec.get(0), true);
            assert_eq(bool_vec.get(1), false);
        }
    }

    mod trait_from {
        use crate::collections::bounded_vec::BoundedVec;
        use crate::convert::From;

        #[test]
        fn simple() {
            let array = [1, 2];
            let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from(array);

            assert_eq(bounded_vec.max_len(), 10);
            assert_eq(bounded_vec.len(), 2);
            assert_eq(bounded_vec.get(0), 1);
            assert_eq(bounded_vec.get(1), 2);
        }
    }

    mod trait_eq {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn empty_equality() {
            let mut bounded_vec1: BoundedVec<Field, 3> = BoundedVec::new();
            let mut bounded_vec2: BoundedVec<Field, 3> = BoundedVec::new();

            assert_eq(bounded_vec1, bounded_vec2);
        }

        #[test]
        fn inequality() {
            let mut bounded_vec1: BoundedVec<Field, 3> = BoundedVec::new();
            let mut bounded_vec2: BoundedVec<Field, 3> = BoundedVec::new();
            bounded_vec1.push(1);
            bounded_vec2.push(2);

            assert(bounded_vec1 != bounded_vec2);
        }
    }

    mod from_parts {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn from_parts() {
            // docs:start:from-parts
            let vec: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 0], 3);
            assert_eq(vec.len(), 3);

            // Any elements past the given length are zeroed out, so these
            // two BoundedVecs will be completely equal
            let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 1], 3);
            let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 2], 3);
            assert_eq(vec1, vec2);
            // docs:end:from-parts
        }

        #[test]
        fn from_parts_unchecked() {
            // docs:start:from-parts-unchecked
            let vec: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 0], 3);
            assert_eq(vec.len(), 3);

            // invalid use!
            let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 1], 3);
            let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 2], 3);

            // both vecs have length 3 so we'd expect them to be equal, but this
            // fails because elements past the length are still checked in eq
            assert(vec1 != vec2);
            // docs:end:from-parts-unchecked
        }
    }

    mod push_pop {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn push_and_pop_operations() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();

            assert_eq(vec.len(), 0);

            vec.push(1);
            assert_eq(vec.len(), 1);
            assert_eq(vec.get(0), 1);

            vec.push(2);
            assert_eq(vec.len(), 2);
            assert_eq(vec.get(1), 2);

            let popped = vec.pop();
            assert_eq(popped, 2);
            assert_eq(vec.len(), 1);

            let popped2 = vec.pop();
            assert_eq(popped2, 1);
            assert_eq(vec.len(), 0);
        }

        #[test(should_fail_with = "push out of bounds")]
        fn push_to_full_vector() {
            let mut vec: BoundedVec<u32, 2> = BoundedVec::new();
            vec.push(1);
            vec.push(2);
            vec.push(3); // should panic
        }

        #[test(should_fail_with = "cannot pop from an empty vector")]
        fn pop_from_empty_vector() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            let _ = vec.pop(); // should panic
        }

        #[test]
        fn push_pop_cycle() {
            let mut vec: BoundedVec<u32, 3> = BoundedVec::new();

            // push to full
            vec.push(1);
            vec.push(2);
            vec.push(3);
            assert_eq(vec.len(), 3);

            // pop all
            assert_eq(vec.pop(), 3);
            assert_eq(vec.pop(), 2);
            assert_eq(vec.pop(), 1);
            assert_eq(vec.len(), 0);

            // push again
            vec.push(4);
            assert_eq(vec.len(), 1);
            assert_eq(vec.get(0), 4);
        }
    }

    mod extend {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn extend_from_array() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            vec.push(1);
            vec.extend_from_array([2, 3]);

            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
        }

        #[test]
        fn extend_from_slice() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            vec.push(1);
            vec.extend_from_slice(&[2, 3]);

            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
        }

        #[test]
        fn extend_from_bounded_vec() {
            let mut vec1: BoundedVec<u32, 5> = BoundedVec::new();
            let mut vec2: BoundedVec<u32, 3> = BoundedVec::new();

            vec1.push(1);
            vec2.push(2);
            vec2.push(3);

            vec1.extend_from_bounded_vec(vec2);

            assert_eq(vec1.len(), 3);
            assert_eq(vec1.get(0), 1);
            assert_eq(vec1.get(1), 2);
            assert_eq(vec1.get(2), 3);
        }

        #[test(should_fail_with = "extend_from_array out of bounds")]
        fn extend_array_beyond_max_len() {
            let mut vec: BoundedVec<u32, 3> = BoundedVec::new();
            vec.push(1);
            vec.extend_from_array([2, 3, 4]); // should panic
        }

        #[test(should_fail_with = "extend_from_slice out of bounds")]
        fn extend_slice_beyond_max_len() {
            let mut vec: BoundedVec<u32, 3> = BoundedVec::new();
            vec.push(1);
            vec.extend_from_slice(&[2, 3, 4]); // S]should panic
        }

        #[test(should_fail_with = "extend_from_bounded_vec out of bounds")]
        fn extend_bounded_vec_beyond_max_len() {
            let mut vec: BoundedVec<u32, 3> = BoundedVec::new();
            let other: BoundedVec<u32, 5> = BoundedVec::from_array([1, 2, 3, 4, 5]);
            vec.extend_from_bounded_vec(other); // should panic
        }

        #[test]
        fn extend_with_empty_collections() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
            let original_len = vec.len();

            vec.extend_from_array([]);
            assert_eq(vec.len(), original_len);

            vec.extend_from_slice(&[]);
            assert_eq(vec.len(), original_len);

            let empty: BoundedVec<u32, 3> = BoundedVec::new();
            vec.extend_from_bounded_vec(empty);
            assert_eq(vec.len(), original_len);
        }
    }

    mod storage {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn storage_consistency() {
            let mut vec: BoundedVec<u32, 5> = BoundedVec::new();

            // test initial storage state
            assert_eq(vec.storage(), [0, 0, 0, 0, 0]);

            vec.push(1);
            vec.push(2);

            // test storage after modifications
            assert_eq(vec.storage(), [1, 2, 0, 0, 0]);

            // storage doesn't change length
            assert_eq(vec.len(), 2);
            assert_eq(vec.max_len(), 5);
        }

        #[test]
        fn storage_after_pop() {
            let mut vec: BoundedVec<u32, 3> = BoundedVec::from_array([1, 2, 3]);

            let _ = vec.pop();
            // after pop, the last element should be zeroed
            assert_eq(vec.storage(), [1, 2, 0]);
            assert_eq(vec.len(), 2);
        }

        #[test]
        fn vector_immutable() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([1, 2, 3]);
            let storage = vec.storage();

            assert_eq(storage, [1, 2, 3]);

            // Verify that the original vector is unchanged
            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
        }
    }
}
