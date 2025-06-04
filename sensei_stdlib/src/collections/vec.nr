pub struct Vec<T> {
    pub(crate) slice: [T],
}
// A mutable vector type implemented as a wrapper around immutable slices.
// A separate type is technically not needed but helps differentiate which operations are mutable.
impl<T> Vec<T> {
    pub fn new() -> Self {
        Self { slice: &[] }
    }

    // Create a Vec containing each element from the given slice.
    // Mutations to the resulting Vec will not affect the original slice.
    pub fn from_slice(slice: [T]) -> Self {
        Self { slice }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index
    /// points beyond the end of the vector.
    pub fn get(self, index: u32) -> T {
        self.slice[index]
    }

    /// Write an element to the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn set(&mut self, index: u32, value: T) {
        self.slice[index] = value;
    }

    /// Push a new element to the end of the vector, returning a
    /// new vector with a length one greater than the
    /// original unmodified vector.
    pub fn push(&mut self, elem: T) {
        self.slice = self.slice.push_back(elem);
    }

    /// Pop an element from the end of the given vector, returning
    /// a new vector with a length of one less than the given vector,
    /// as well as the popped element.
    /// Panics if the given vector's length is zero.
    pub fn pop(&mut self) -> T {
        let (popped_slice, last_elem) = self.slice.pop_back();
        self.slice = popped_slice;
        last_elem
    }

    /// Insert an element at a specified index, shifting all elements
    /// after it to the right
    pub fn insert(&mut self, index: u32, elem: T) {
        self.slice = self.slice.insert(index, elem);
    }

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the removed element
    pub fn remove(&mut self, index: u32) -> T {
        let (new_slice, elem) = self.slice.remove(index);
        self.slice = new_slice;
        elem
    }

    /// Returns the number of elements in the vector
    pub fn len(self) -> u32 {
        self.slice.len()
    }
}

mod tests {
    use crate::collections::vec::Vec;

    #[test]
    fn set_updates_values_properly() {
        let mut vec = Vec { slice: &[0, 0, 0, 0, 0] };

        vec.set(0, 42);
        assert_eq(vec.slice, &[42, 0, 0, 0, 0]);

        vec.set(1, 43);
        assert_eq(vec.slice, &[42, 43, 0, 0, 0]);

        vec.set(2, 44);
        assert_eq(vec.slice, &[42, 43, 44, 0, 0]);

        vec.set(1, 10);
        assert_eq(vec.slice, &[42, 10, 44, 0, 0]);

        vec.set(0, 0);
        assert_eq(vec.slice, &[0, 10, 44, 0, 0]);
    }

    #[test(should_fail)]
    fn panics_when_writing_elements_past_end_of_vec() {
        let mut vec = Vec::new();
        vec.set(0, 42);

        // Need to use println to avoid DIE removing the write operation.
        crate::println(vec.get(0));
    }
}
