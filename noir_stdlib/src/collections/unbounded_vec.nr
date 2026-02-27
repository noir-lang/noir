use crate::{cmp::Eq, collections::bounded_vec::BoundedVec, convert::From, runtime::is_unconstrained, static_assert};


#[builtin(resize_array)]
fn resize_array<T>(from: [T], adjust: i32) -> [T]{}

#[builtin(vector_enumerate)]
pub fn vector_enumerate<T>(vec: [T], f: fn([T], u32, u32) -> T, index: u32, rev: bool) -> [T] {}

pub struct UnboundedVec<T> {
   pub storage: [T],
   pub len: u32,
}

impl<T> UnboundedVec<T> {

    /// Creates a new, empty vector of length zero.
    ///
    /// Since this container is backed by a vector internally, it still needs an initial value
    /// to give each element. To resolve this, each element is zeroed internally. This value
    /// is guaranteed to be inaccessible unless `get_unchecked` is used.
    ///
    /// Example:
    ///
    /// ```noir
    /// let empty_vector: UnboundedVec<Field> = UnboundedVec::new();
    /// assert(empty_vector.len() == 0);
    /// ```
    ///
    pub fn new() -> Self {
        let zeroed = crate::mem::zeroed();
        UnboundedVec { storage: [zeroed; 0].as_vector(), len: 0}
    }

    /// Retrieves an element from the vector at the given index, starting from zero.
    ///
    /// If the given index is equal to or greater than the length of the vector, this
    /// will issue a constraint failure.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn foo(v: UnboundedVec<u32>) {
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
    /// fn sum_of_first_three(v: UnboundedVec<u32>) -> u32 {
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
    /// fn foo(v: UnboundedVec<u32>) {
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


pub fn len(self) -> u32 {
        self.len
    }
   /// Writes an element to the vector at the given index, starting from zero, without performing a bounds check.
    ///
    /// Since this function does not perform a bounds check on length before accessing the element, it is unsafe! Use at your own risk!
    ///
    /// Example:
    ///
    /// ```noir
    /// fn set_unchecked_example() {
    ///     let mut vec: BoundedVec<u32> = BoundedVec::new();
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
    /// v.push_back(3);
    /// ```
    pub fn push_back(&mut self, elem: T) {
        self.storage = resize_array(self.storage, 1);
        self.set_unchecked(self.len, elem);
        self.len += 1;

    }

    pub fn push_front(&mut self, elem: T)  where T: Eq {
        self.insert(0, elem);
    }

    pub fn insert(&mut self, index: u32, elem: T) where T: Eq {
        assert(index <= self.len, "Attempted to insert past end of UnboundedVec");
        self.storage = resize_array(self.storage, 1);
        let mut offset = 1;

        self.storage = vector_enumerate(self.storage, |vec: [T], i, idx| -> T {
        if i > idx {
            vec[i-1]
        } else {
            vec[i]
        }
   }, index, true);  // Reverse iteration
       self.set_unchecked(index, elem);
        self.len += 1;
    }

    pub fn remove(&mut self, index: u32) -> T where T: Eq {
        assert(index < self.len, "Attempted to remove past end of BoundedVec");
        let elem = self.storage[index];
        let original_len = self.len;

        // Enumerate BEFORE resizing so we can access all n values of the original vector
        // Pass index (removal position) as the user_index parameter
        // At the last position (i=n-1), we can't access vec[i+1], so we return vec[i]
        // (which creates a duplicate that will be discarded by the subsequent resize)
        self.storage = vector_enumerate(self.storage, |vec: [T], i, idx| -> T {
            if i >= idx {
                if i + 1 < vec.len() {
                    vec[i + 1]
                } else {
                    vec[i]  // Last position: duplicate (will be discarded)
                }
            } else {
                vec[i]
            }
        }, index, false);

        // Resize after enumerate to discard the last position
        self.storage = resize_array(self.storage, -1);
        self.len -= 1;

        elem
    }

    /// Returns the maximum length of this vector.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut v: UnboundedVec<Field> = UnboundedVec::new();
    ///
    /// v.push(10);
    /// assert(v.max_len() == 1);
    /// ```
    pub fn max_len(self) -> u32 {
        self.storage.len()
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
    /// let mut v: UnboundedVec<Field> = UnboundedVec::new();
    ///
    /// v.push(57);
    /// assert(v.storage() == [57]);
    /// ```
    pub fn storage(self) -> [T] {
        self.storage
    }

    /// Pushes each element from the given array to this vector.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut vec: UnboundedVec<Field, 3> = UnboundedVec::new();
    /// vec.extend_from_array([2, 4]);
    ///
    /// assert(vec.len == 2);
    /// assert(vec.get(0) == 2);
    /// assert(vec.get(1) == 4);
    /// ```
    pub fn extend_from_array<let Len: u32>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();

        // if new_len > self.storage.len() {
        //     self.resize(new_len);
        // }
        self.storage = resize_array(self.storage, Len as i32);
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
        // self.max_len = Len;
    }

    /// Pushes each element from the given vector to this vector.
    ///
    /// Panics if pushing each element would cause the length of this vector
    /// to exceed the maximum length.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut vec: UnboundedVec<Field> = UnboundedVec::new();
    /// vec.extend_from_vector([2, 4].as_vector());
    ///
    /// assert(vec.len == 2);
    /// assert(vec.get(0) == 2);
    /// assert(vec.get(1) == 4);
    /// ```
    pub fn extend_from_vector(&mut self, vector: [T]) {
        let new_len = self.len + vector.len();

        // if new_len > self.storage.len() {
        //     self.resize(new_len);
        // }
            self.storage = resize_array(self.storage, vector.len() as i32);
        for i in 0..vector.len() {
            self.storage[self.len + i] = vector[i];
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
    /// let mut v1: UnboundedVec<Field, 5> = UnboundedVec::new();
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

        // if new_len > self.storage.len() {
        //     self.resize(new_len);
        // }
        self.storage = resize_array(self.storage, Len as i32);


           for i in 0..append_len {
                self.storage[self.len + i] = vec.get_unchecked(i);
            }
        // if is_unconstrained() {
        //     for i in 0..append_len {
        //         self.storage[self.len + i] = vec.get_unchecked(i);
        //     }
        // } else {
        //     for i in 0..Len {
        //         if i < append_len {
        //             self.storage[self.len + i] = vec.get_unchecked(i);
        //         }
        //     }
        // }
        self.len = new_len;
    }
//TODO: extend_from_unbounded_vec

    /// Creates a new vector, populating it with values derived from an array input.
    ///
    /// Example:
    ///
    /// ```noir
    /// let bounded_vec: UnboundedVec<Field> = UnboundedVec::from_array([1, 2, 3])
    /// ```
    pub fn from_array<let Len: u32>(array: [T; Len]) -> Self {
        let mut vec: UnboundedVec<T> = UnboundedVec::new();
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
    /// let mut v: UnboundedVec<Field, 2> = UnboundedVec::new();
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
    pub fn pop_back(&mut self) -> T {
        assert(self.len > 0, "cannot pop from an empty vector");
        self.len -= 1;
 
        let elem = self.storage[self.len];
        self.storage = resize_array(self.storage, -1);
        //self.max_len -= 1;
        //self.storage[self.len] = crate::mem::zeroed();
        elem
    }

    pub fn pop_front(&mut self) -> T  where T: Eq {
        self.remove(0)
        // assert(self.len > 0, "cannot pop from an empty vector");
        // self.len -= 1;
        // let elem = self.storage[0];
        // for i in 0..self.storage.len()-1 {
        //     self.storage[i] = self.storage[i+1];
        // }
        // self.storage = resize_array(self.storage, -1);
        // elem
    }
}

impl<T> Eq for UnboundedVec<T>
where
    T: Eq,
{
    fn eq(self, other: UnboundedVec<T>) -> bool {
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


mod unbounded_vec_tests {

    mod get {
        use crate::collections::unbounded_vec::UnboundedVec;

        #[test(should_fail_with = "Index out of bounds")]
        fn panics_when_reading_elements_past_end_of_vec() {
            let vec: UnboundedVec<Field> = UnboundedVec::new();

            let _ = vec.get(0);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn panics_when_reading_beyond_length() {
            let vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3]);
            let _ = vec.get(3);
        }

        #[test]
        fn get_works_within_bounds() {
            let vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3, 4, 5]);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(4), 5);
        }

           #[test]
        fn get_unchecked_works() {
            let vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3]);
            assert_eq(vec.get_unchecked(0), 1);
            assert_eq(vec.get_unchecked(2), 3);
        }

        // #[test]
        // fn get_unchecked_works_past_len() {
        //     let vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3]);
        //     assert_eq(vec.get_unchecked(4), 0);
        // }
    }

      mod set {
        use crate::collections::unbounded_vec::UnboundedVec;

        #[test]
        fn set_updates_values_properly() {
            let mut vec = UnboundedVec::from_array([0, 0, 0, 0, 0]);
//TODO !!
            vec.set(0, 42);
            assert_eq(vec.storage, [42, 0, 0, 0, 0].as_vector());

            vec.set(1, 43);
            assert_eq(vec.storage, [42, 43, 0, 0, 0].as_vector());

            vec.set(2, 44);
            assert_eq(vec.storage, [42, 43, 44, 0, 0].as_vector());

            vec.set(1, 10);
            assert_eq(vec.storage, [42, 10, 44, 0, 0].as_vector());

            vec.set(0, 0);
            assert_eq(vec.storage, [0, 10, 44, 0, 0].as_vector());
        }

        #[test(should_fail_with = "Attempted to write past end of BoundedVec")]
        fn panics_when_writing_elements_past_end_of_vec() {
            let mut vec: UnboundedVec<Field> = UnboundedVec::new();
            vec.set(0, 42);
        }

        #[test(should_fail_with = "Attempted to write past end of BoundedVec")]
        fn panics_when_setting_beyond_length() {
            let mut vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3]);
            vec.set(3, 4);
        }

        #[test]
        fn set_unchecked_operations() {
            let mut vec: UnboundedVec<u32> = UnboundedVec::new();
            vec.push_back(1);
            vec.push_back(2);

            vec.set_unchecked(0, 10);
            assert_eq(vec.get(0), 10);
        }

        #[test(should_fail_with = "Attempted to read past end of BoundedVec")]
        fn set_unchecked_operations_past_len() {
            let mut vec: UnboundedVec<u32> = UnboundedVec::new();
            vec.push_back(1);
            vec.push_back(2);
//TODO
            vec.set_unchecked(3, 40);
            assert_eq(vec.get(3), 40);
        }

        #[test]
        fn set_preserves_other_elements() {
            let mut vec: UnboundedVec<u32> = UnboundedVec::from_array([1, 2, 3, 4, 5]);

            vec.set(2, 30);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 30);
            assert_eq(vec.get(3), 4);
            assert_eq(vec.get(4), 5);
        }
    }

}