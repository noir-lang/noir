pub struct Vec<T> {
    pub(crate) list: [T],
}
// A mutable vector type implemented as a wrapper around immutable lists.
// A separate type is technically not needed but helps differentiate which operations are mutable.
impl<T> Vec<T> {
    pub fn new() -> Self {
        Self { list: &[] }
    }

    // Create a Vec containing each element from the given list.
    // Mutations to the resulting Vec will not affect the original list.
    pub fn from_list(list: [T]) -> Self {
        Self { list }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index
    /// points beyond the end of the vector.
    pub fn get(self, index: u32) -> T {
        self.list[index]
    }

    /// Write an element to the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn set(&mut self, index: u32, value: T) {
        self.list[index] = value;
    }

    /// Push a new element to the end of the vector, returning a
    /// new vector with a length one greater than the
    /// original unmodified vector.
    pub fn push(&mut self, elem: T) {
        self.list = self.list.push_back(elem);
    }

    /// Pop an element from the end of the given vector, returning
    /// a new vector with a length of one less than the given vector,
    /// as well as the popped element.
    /// Panics if the given vector's length is zero.
    pub fn pop(&mut self) -> T {
        let (popped_list, last_elem) = self.list.pop_back();
        self.list = popped_list;
        last_elem
    }

    /// Insert an element at a specified index, shifting all elements
    /// after it to the right
    pub fn insert(&mut self, index: u32, elem: T) {
        self.list = self.list.insert(index, elem);
    }

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the removed element
    pub fn remove(&mut self, index: u32) -> T {
        let (new_list, elem) = self.list.remove(index);
        self.list = new_list;
        elem
    }

    /// Returns the number of elements in the vector
    pub fn len(self) -> u32 {
        self.list.len()
    }
}

mod vec_tests {
    mod new {
        use crate::collections::vec::Vec;

        #[test]
        fn creates_empty_vector() {
            let vec: Vec<Field> = Vec::new();
            assert_eq(vec.len(), 0);
        }
    }

    mod from_list {
        use crate::collections::vec::Vec;

        #[test]
        fn creates_vector_with_elements() {
            let list = &[1, 2, 3, 4, 5];
            let vec = Vec::from_list(list);
            assert_eq(vec.len(), 5);
            assert_eq(vec.list, list);
        }
    }

    mod get {
        use crate::collections::vec::Vec;

        #[test]
        fn retrieves_elements_correctly() {
            let vec = Vec { list: &[10, 20, 30, 40, 50] };

            assert_eq(vec.get(0), 10);
            assert_eq(vec.get(1), 20);
            assert_eq(vec.get(2), 30);
            assert_eq(vec.get(3), 40);
            assert_eq(vec.get(4), 50);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn panics_on_out_of_bounds() {
            let vec = Vec { list: &[1, 2, 3] };
            let _ = vec.get(3);
        }
    }

    mod set {
        use crate::collections::vec::Vec;

        #[test]
        fn updates_values_properly() {
            let mut vec = Vec { list: &[0, 0, 0, 0, 0] };

            vec.set(0, 42);
            assert_eq(vec.list, &[42, 0, 0, 0, 0]);

            vec.set(1, 43);
            assert_eq(vec.list, &[42, 43, 0, 0, 0]);

            vec.set(2, 44);
            assert_eq(vec.list, &[42, 43, 44, 0, 0]);

            vec.set(1, 10);
            assert_eq(vec.list, &[42, 10, 44, 0, 0]);

            vec.set(0, 0);
            assert_eq(vec.list, &[0, 10, 44, 0, 0]);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn panics_when_writing_elements_past_end_of_vec() {
            let mut vec = Vec::new();
            vec.set(0, 42);

            let _ = vec.get(0);
        }
    }

    mod push {
        use crate::collections::vec::Vec;

        #[test]
        fn adds_element_to_end() {
            let mut vec = Vec { list: &[1, 2, 3] };
            let original_len = vec.len();

            vec.push(4);

            assert_eq(vec.len(), original_len + 1);
            assert_eq(vec.get(3), 4);
        }

        #[test]
        fn on_empty_vector() {
            let mut vec = Vec::new();
            vec.push(42);

            assert_eq(vec.len(), 1);
            assert_eq(vec.get(0), 42);
        }
    }

    mod pop {
        use crate::collections::vec::Vec;

        #[test]
        fn removes_last_element() {
            let mut vec = Vec { list: &[1, 2, 3, 4, 5] };
            let original_len = vec.len();

            let popped = vec.pop();

            assert_eq(popped, 5);
            assert_eq(vec.len(), original_len - 1);
            assert_eq(vec.get(3), 4);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn on_empty_vector_panics() {
            let mut vec: Vec<Field> = Vec::new();
            let _ = vec.pop();
        }
    }

    mod insert {
        use crate::collections::vec::Vec;

        #[test]
        fn at_beginning() {
            let mut vec = Vec { list: &[1, 2, 3] };
            vec.insert(0, 0);

            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 0);
            assert_eq(vec.get(1), 1);
            assert_eq(vec.get(2), 2);
            assert_eq(vec.get(3), 3);
        }

        #[test]
        fn at_middle() {
            let mut vec = Vec { list: &[1, 2, 4, 5] };
            vec.insert(2, 3);

            assert_eq(vec.len(), 5);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(3), 4);
            assert_eq(vec.get(4), 5);
        }

        #[test]
        fn at_end() {
            let mut vec = Vec { list: &[1, 2, 3] };
            vec.insert(3, 4);

            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(3), 4);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn beyond_length_panics() {
            let mut vec = Vec { list: &[1, 2, 3] };
            vec.insert(4, 5);
        }
    }

    mod remove {
        use crate::collections::vec::Vec;

        #[test]
        fn from_beginning() {
            let mut vec = Vec { list: &[0, 1, 2, 3, 4] };
            let removed = vec.remove(0);

            assert_eq(removed, 0);
            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(3), 4);
        }

        #[test]
        fn from_middle() {
            let mut vec = Vec { list: &[1, 2, 3, 4, 5] };
            let removed = vec.remove(2);

            assert_eq(removed, 3);
            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 4);
            assert_eq(vec.get(3), 5);
        }

        #[test]
        fn from_end() {
            let mut vec = Vec { list: &[1, 2, 3, 4, 5] };
            let removed = vec.remove(4);

            assert_eq(removed, 5);
            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);
            assert_eq(vec.get(3), 4);
        }

        #[test(should_fail_with = "Index out of bounds")]
        fn beyond_length_panics() {
            let mut vec = Vec { list: &[1, 2, 3] };
            let _ = vec.remove(3);
        }
    }

    mod len {
        use crate::collections::vec::Vec;

        #[test]
        fn returns_correct_length() {
            let empty_vec: Vec<Field> = Vec::new();
            assert_eq(empty_vec.len(), 0);

            let single_vec = Vec { list: &[42] };
            assert_eq(single_vec.len(), 1);

            let multi_vec = Vec { list: &[1, 2, 3, 4, 5] };
            assert_eq(multi_vec.len(), 5);
        }
    }

    mod vec_integration {
        use crate::collections::vec::Vec;

        #[test]
        fn multiple_operations_chain_correctly() {
            let mut vec = Vec::new();

            // push several elements
            vec.push(1);
            vec.push(2);
            vec.push(3);
            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 2);
            assert_eq(vec.get(2), 3);

            // insert in middle
            vec.insert(1, 10);
            assert_eq(vec.len(), 4);
            assert_eq(vec.get(0), 1);
            assert_eq(vec.get(1), 10);
            assert_eq(vec.get(2), 2);
            assert_eq(vec.get(3), 3);

            // set elements
            vec.set(0, 100);
            vec.set(2, 200);
            assert_eq(vec.get(0), 100);
            assert_eq(vec.get(2), 200);

            // remove element
            let removed = vec.remove(1);
            assert_eq(removed, 10);
            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), 100);
            assert_eq(vec.get(1), 200);
            assert_eq(vec.get(2), 3);

            // pop element
            let popped = vec.pop();
            assert_eq(popped, 3);
            assert_eq(vec.len(), 2);
        }

        #[test]
        fn vector_with_custom_types() {
            let mut vec = Vec { list: &[true, false, true] };

            assert_eq(vec.len(), 3);
            assert_eq(vec.get(0), true);
            assert_eq(vec.get(1), false);
            assert_eq(vec.get(2), true);

            vec.set(1, true);
            assert_eq(vec.get(1), true);

            vec.push(false);
            assert_eq(vec.len(), 4);
            assert_eq(vec.get(3), false);
        }
    }
}
