use crate::{cmp::Eq, convert::From};

struct BoundedVec<T, let MaxLen: u32> {
    storage: [T; MaxLen],
    len: u32,
}

impl<T, let MaxLen: u32> BoundedVec<T, MaxLen> {
    pub fn new() -> Self {
        let zeroed = crate::unsafe::zeroed();
        BoundedVec { storage: [zeroed; MaxLen], len: 0 }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn get(self, index: u32) -> T {
        assert(index < self.len, "Attempted to read past end of BoundedVec");
        self.get_unchecked(index)
    }

    /// Get an element from the vector at the given index.
    /// Responds with undefined data for `index` where `self.len < index < self.max_len()`.
    pub fn get_unchecked(self, index: u32) -> T {
        self.storage[index]
    }

    /// Write an element to the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn set(&mut self, index: u32, value: T) {
        assert(index < self.len, "Attempted to write past end of BoundedVec");
        self.set_unchecked(index, value)
    }

    /// Write an element to the vector at the given index.
    /// Does not check whether the passed `index` is a valid index within the vector.
    ///
    /// Silently writes past the end of the vector for `index` where `self.len < index < self.max_len()`
    /// Panics if the given index points beyond the maximum length of the vector (`self.max_len()`).
    pub fn set_unchecked(&mut self, index: u32, value: T) {
        self.storage[index] = value;
    }

    pub fn push(&mut self, elem: T) {
        assert(self.len < MaxLen, "push out of bounds");

        self.storage[self.len] = elem;
        self.len += 1;
    }

    pub fn len(self) -> u32 {
        self.len
    }

    pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u32 {
        MaxLen
    }

    // This is a intermediate method, while we don't have an
    // .extend method
    pub fn storage(self) -> [T; MaxLen] {
        self.storage
    }

    pub fn extend_from_array<let Len: u32>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();
        assert(new_len <= MaxLen, "extend_from_array out of bounds");
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_slice(&mut self, slice: [T]) {
        let new_len = self.len + slice.len();
        assert(new_len <= MaxLen, "extend_from_slice out of bounds");
        for i in 0..slice.len() {
            self.storage[self.len + i] = slice[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_bounded_vec<let Len: u32>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len <= MaxLen, "extend_from_bounded_vec out of bounds");

        let mut exceeded_len = false;
        for i in 0..Len {
            exceeded_len |= i == append_len;
            if !exceeded_len {
                self.storage[self.len + i] = vec.get_unchecked(i);
            }
        }
        self.len = new_len;
    }

    pub fn from_array<let Len: u32>(array: [T; Len]) -> Self {
        assert(Len <= MaxLen, "from array out of bounds");
        let mut vec: BoundedVec<T, MaxLen> = BoundedVec::new();
        vec.extend_from_array(array);
        vec
    }

    pub fn pop(&mut self) -> T {
        assert(self.len > 0);
        self.len -= 1;

        let elem = self.storage[self.len];
        self.storage[self.len] = crate::unsafe::zeroed();
        elem
    }

    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        let mut exceeded_len = false;
        for i in 0..MaxLen {
            exceeded_len |= i == self.len;
            if !exceeded_len {
                ret |= predicate(self.storage[i]);
            }
        }
        ret
    }

    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> BoundedVec<U, MaxLen> {
        let mut ret = BoundedVec::new();
        ret.len = self.len();
        for i in 0..MaxLen {
            if i < self.len() {
                ret.storage[i] = f(self.get_unchecked(i));
            }
        }
        ret
    }
}

impl<T, let MaxLen: u32> Eq for BoundedVec<T, MaxLen> where T: Eq {
    fn eq(self, other: BoundedVec<T, MaxLen>) -> bool {
        // TODO: https://github.com/noir-lang/noir/issues/4837
        //
        // We make the assumption that the user has used the proper interface for working with `BoundedVec`s
        // rather than directly manipulating the internal fields as this can result in an inconsistent internal state.
        
        (self.len == other.len) & (self.storage == other.storage)
    }
}

impl<T, let MaxLen: u32, let Len: u32> From<[T; Len]> for BoundedVec<T, MaxLen> {
    fn from(array: [T; Len]) -> BoundedVec<T, MaxLen>  {
        BoundedVec::from_array(array)
    }
}

mod bounded_vec_tests {

    mod get {
        use crate::collections::bounded_vec::BoundedVec;

        #[test(should_fail_with = "Attempted to read past end of BoundedVec")]
        fn panics_when_reading_elements_past_end_of_vec() {
            let vec: BoundedVec<Field, 5> = BoundedVec::new();

            crate::println(vec.get(0));
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

            // Need to use println to avoid DIE removing the write operation.
            crate::println(vec.get(0));
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
            let result = vec.map(|value| (value * 2)  as Field);
            let expected: BoundedVec<Field, 4> = BoundedVec::from_array([2, 4, 6, 8]);

            assert_eq(result, expected);
        }

        #[test]
        fn does_not_apply_function_past_len() {
            let vec: BoundedVec<u32, 3> = BoundedVec::from_array([0, 1]);
            let result = vec.map(|value| if value == 0 { 5 } else { value });
            let expected = BoundedVec::from_array([5, 1]);

            assert_eq(result, expected);
            assert_eq(result.storage()[2], 0);
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
            assert_eq(bounded_vec.storage()[0], 1);
            assert_eq(bounded_vec.storage()[1], 2);
            assert_eq(bounded_vec.storage()[2], 3);
        }

        #[test(should_fail_with="from array out of bounds")]
        fn max_len_lower_then_array_len() {
            let _: BoundedVec<Field, 2> = BoundedVec::from_array([0; 3]);
        }
    }

    mod trait_from {
        use crate::collections::bounded_vec::BoundedVec;

        #[test]
        fn simple() {
            let array = [1, 2];
            let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from(array);

            assert_eq(bounded_vec.max_len(), 10);
            assert_eq(bounded_vec.len(), 2);
            assert_eq(bounded_vec.storage()[0], 1);
            assert_eq(bounded_vec.storage()[1], 2);
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
}
