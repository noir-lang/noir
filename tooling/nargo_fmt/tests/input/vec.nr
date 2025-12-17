struct Vec<T> { 
    list: [T]
}

// A mutable vector type implemented as a wrapper around immutable lists.
// A separate type is technically not needed but helps differentiate which operations are mutable.
impl<T> Vec<T> {
    pub fn new() -> Self {
        Self { list: [] }
    }

    // Create a Vec containing each element from the given list.
    // Mutations to the resulting Vec will not affect the original list.
    pub fn from_list(list: [T]) -> Self {
        Self { list }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index
    /// points beyond the end of the vector.
    pub fn get(self, index: Field) -> T {
        self.list[index]
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
    pub fn insert(&mut self, index: Field, elem: T) {
        self.list = self.list.insert(index, elem);
    } 

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the removed element
    pub fn remove(&mut self, index: Field) -> T {
        let (new_list, elem) = self.list.remove(index);
        self.list = new_list;
        elem
    }

    /// Returns the number of elements in the vector
    pub fn len(self) -> Field {
        self.list.len()
    }
}
