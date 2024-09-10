# Module `std::collections::vec`

### Methods

#### new

```noir
fn new() -> Self
```

#### from_slice

```noir
fn from_slice(slice: [T]) -> Self
```

#### get

```noir
fn get(self, index: u32) -> T
```

Get an element from the vector at the given index.
Panics if the given index
points beyond the end of the vector.

#### set

```noir
fn set(self, index: u32, value: T)
```

Write an element to the vector at the given index.
Panics if the given index points beyond the end of the vector (`self.len()`).

#### push

```noir
fn push(self, elem: T)
```

Push a new element to the end of the vector, returning a
new vector with a length one greater than the
original unmodified vector.

#### pop

```noir
fn pop(self) -> T
```

Pop an element from the end of the given vector, returning
a new vector with a length of one less than the given vector,
as well as the popped element.
Panics if the given vector's length is zero.

#### insert

```noir
fn insert(self, index: u32, elem: T)
```

Insert an element at a specified index, shifting all elements
after it to the right

#### remove

```noir
fn remove(self, index: u32) -> T
```

Remove an element at a specified index, shifting all elements
after it to the left, returning the removed element

#### len

```noir
fn len(self) -> u32
```

Returns the number of elements in the vector

