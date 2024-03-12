---
title: Bounded Vectors
keywords: [noir, vector, bounded vector, slice]
sidebar_position: 1
---

A `BoundedVec<T, MaxLen>` is a growable storage similar to a `Vec<T>` except that it
is bounded with a maximum possible length. Unlike `Vec`, `BoundedVec` is not implemented
via slices and thus is not subject to the same restrictions slices are (notably, nested
slices - and thus nested vectors as well - are disallowed).

Since a BoundedVec is backed by a normal array under the hood, growing the BoundedVec by
pushing an additional element is also more efficient - the length only needs to be increased
by one.

For these reasons `BoundedVec<T, N>` should generally be preferred over `Vec<T>` when there
is a reasonable maximum bound that can be placed on the vector.

Example:

```rust
let mut vector: BoundedVec<Field, 10> = BoundedVec::new();
for i in 0..5 {
    vector.push(i);
}
assert(vector.len() == 5);
assert(vector.max_len() == 10);
```

## Methods

### new

```rust
pub fn new() -> Self
```

Creates a new, empty vector of length zero.

Since this container is backed by an array internally, it still needs an initial value
to give each element. To resolve this, each element is zeroed internally. This value
is guaranteed to be inaccessible unless `get_unchecked` is used.

Example:

```rust
let empty_vector: BoundedVec<Field, 10> = BoundedVec::new();
assert(empty_vector.len() == 0);
```

Note that whenever calling `new` the maximum length of the vector should always be specified
via a type signature:

```rust title="new_example" showLineNumbers 
fn foo() -> BoundedVec<Field, 10> {
    // Ok! MaxLen is specified with a type annotation
    let v1: BoundedVec<Field, 3> = BoundedVec::new();
    let v2 = BoundedVec::new();

    // Ok! MaxLen is known from the type of foo's return value
    v2
}

fn bad() {
    let mut v3 = BoundedVec::new();

    // Not Ok! We don't know if v3's MaxLen is at least 1, and the compiler often infers 0 by default.
    v3.push(5);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L11-L27" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L11-L27</a></sub></sup>


This defaulting of `MaxLen` (and numeric generics in general) to zero may change in future noir versions
but for now make sure to use type annotations when using bounded vectors. Otherwise, you will receive a constraint failure at runtime when the vec is pushed to.

### get

```rust
pub fn get(mut self: Self, index: u64) -> T {
```

Retrieves an element from the vector at the given index, starting from zero.

If the given index is equal to or greater than the length of the vector, this
will issue a constraint failure.

Example:

```rust
fn foo<N>(v: BoundedVec<u32, N>) {
    let first = v.get(0);
    let last = v.get(v.len() - 1);
    assert(first != last);
}
```

### get_unchecked

```rust
pub fn get_unchecked(mut self: Self, index: u64) -> T {
```

Retrieves an element from the vector at the given index, starting from zero, without
performing a bounds check.

Since this function does not perform a bounds check on length before accessing the element,
it is unsafe! Use at your own risk!

Example:

```rust title="get_unchecked_example" showLineNumbers 
fn sum_of_first_three<N>(v: BoundedVec<u32, N>) -> u32 {
    // Always ensure the length is larger than the largest
    // index passed to get_unchecked
    assert(v.len() > 2);
    let first = v.get_unchecked(0);
    let second = v.get_unchecked(1);
    let third = v.get_unchecked(2);
    first + second + third
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L54-L64" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L54-L64</a></sub></sup>



### push

```rust
pub fn push(&mut self, elem: T) {
```

Pushes an element to the end of the vector. This increases the length
of the vector by one.

Panics if the new length of the vector will be greater than the max length.

Example:

```rust title="bounded-vec-push-example" showLineNumbers 
let mut v: BoundedVec<Field, 2> = BoundedVec::new();

    v.push(1);
    v.push(2);

    // Panics with failed assertion "push out of bounds"
    v.push(3);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L68-L76" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L68-L76</a></sub></sup>


### pop

```rust
pub fn pop(&mut self) -> T
```

Pops the element at the end of the vector. This will decrease the length
of the vector by one.

Panics if the vector is empty.

Example:

```rust title="bounded-vec-pop-example" showLineNumbers 
let mut v: BoundedVec<Field, 2> = BoundedVec::new();
    v.push(1);
    v.push(2);

    let two = v.pop();
    let one = v.pop();

    assert(two == 2);
    assert(one == 1);
    // error: cannot pop from an empty vector
    // let _ = v.pop();
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L81-L93" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L81-L93</a></sub></sup>


### len

```rust
pub fn len(self) -> u64 {
```

Returns the current length of this vector

Example:

```rust title="bounded-vec-len-example" showLineNumbers 
let mut v: BoundedVec<Field, 4> = BoundedVec::new();
    assert(v.len() == 0);

    v.push(100);
    assert(v.len() == 1);

    v.push(200);
    v.push(300);
    v.push(400);
    assert(v.len() == 4);

    let _ = v.pop();
    let _ = v.pop();
    assert(v.len() == 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L98-L113" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L98-L113</a></sub></sup>


### max_len

```rust
pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u64 {
```

Returns the maximum length of this vector. This is always
equal to the `MaxLen` parameter this vector was initialized with.

Example:

```rust title="bounded-vec-max-len-example" showLineNumbers 
let mut v: BoundedVec<Field, 5> = BoundedVec::new();

    assert(v.max_len() == 5);
    v.push(10);
    assert(v.max_len() == 5);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L118-L124" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L118-L124</a></sub></sup>


### storage

```rust
pub fn storage(self) -> [T; MaxLen] {
```

Returns the internal array within this vector.
Since arrays in Noir are immutable, mutating the returned storage array will not mutate
the storage held internally by this vector.

Note that uninitialized elements may be zeroed out!

Example:

```rust title="bounded-vec-storage-example" showLineNumbers 
let mut v: BoundedVec<Field, 5> = BoundedVec::new();

    assert(v.storage() == [0, 0, 0, 0, 0]);

    v.push(57);
    assert(v.storage() == [57, 0, 0, 0, 0]);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L129-L136" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L129-L136</a></sub></sup>


### extend_from_array

```rust
pub fn extend_from_array<Len>(&mut self, array: [T; Len])
```

Pushes each element from the given array to this vector.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

```rust title="bounded-vec-extend-from-array-example" showLineNumbers 
let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4]);

    assert(vec.len == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L141-L148" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L141-L148</a></sub></sup>


### extend_from_bounded_vec

```rust
pub fn extend_from_bounded_vec<Len>(&mut self, vec: BoundedVec<T, Len>)
```

Pushes each element from the other vector to this vector. The length of
the other vector is left unchanged.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

```rust title="bounded-vec-extend-from-bounded-vec-example" showLineNumbers 
let mut v1: BoundedVec<Field, 5> = BoundedVec::new();
    let mut v2: BoundedVec<Field, 7> = BoundedVec::new();

    v2.extend_from_array([1, 2, 3]);
    v1.extend_from_bounded_vec(v2);

    assert(v1.storage() == [1, 2, 3, 0, 0]);
    assert(v2.storage() == [1, 2, 3, 0, 0, 0, 0]);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L153-L162" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L153-L162</a></sub></sup>


### any

```rust
pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

Returns true if the given predicate returns true for any element
in this vector.

Example:

```rust title="bounded-vec-any-example" showLineNumbers 
let mut v: BoundedVec<u32, 3> = BoundedVec::new();
    v.extend_from_array([2, 4, 6]);

    let all_even = !v.any(|elem: u32| elem % 2 != 0);
    assert(all_even);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L229-L235" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L229-L235</a></sub></sup>

