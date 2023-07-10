struct Vec<T> { 
    slice: [T]
}

// These methods are all stubs currently and aren't implemented internally yet.
// For a similar reason, no constructor for Vec is exposed yet since the type
// is still in-progress.
impl<T> Vec<T> {
    fn new() -> Self {
        Self { slice: [] }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index
    /// points beyond the end of the vector.
    fn get(&mut self, index: Field) -> T {
        self.slice[index]
     }

    /// Push a new element to the end of the vector, returning a
    /// new vector with a length one greater than the
    /// original unmodified vector.
    fn push(&mut self, elem: T) { 
        self.slice = self.slice.push_back(elem);
    }

    /// Pop an element from the end of the given vector, returning
    /// a new vector with a length of one less than the given vector,
    /// as well as the popped element.
    /// Panics if the given vector's length is zero.
    fn pop(&mut self) -> T { 
        let (popped_slice, last_elem) = self.slice.pop_back();
        self.slice = popped_slice;
        last_elem
    }

    /// Insert an element at a specified index, shifting all elements 
    /// after it to the right
    fn insert(&mut self, index: Field, elem: T) {
        self.slice = self.slice.insert(index, elem);
    } 

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the removed element
    fn remove(&mut self, index: Field) -> T {
        let (new_slice, elem) = self.slice.remove(index);
        self.slice = new_slice;
        elem
    }

    /// Returns the number of elements in the vector
    fn len(self: Self) -> Field {
        self.slice.len()
    }
}