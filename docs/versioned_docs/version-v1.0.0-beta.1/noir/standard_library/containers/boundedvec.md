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
fn good() -> BoundedVec<Field, 10> {
    // Ok! MaxLen is specified with a type annotation
    let v1: BoundedVec<Field, 3> = BoundedVec::new();
    let v2 = BoundedVec::new();

    // Ok! MaxLen is known from the type of `good`'s return value
    v2
}

fn bad() {
    // Error: Type annotation needed
    // The compiler can't infer `MaxLen` from this code.
    let mut v3 = BoundedVec::new();
    v3.push(5);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L11-L27" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L11-L27</a></sub></sup>


This defaulting of `MaxLen` (and numeric generics in general) to zero may change in future noir versions
but for now make sure to use type annotations when using bounded vectors. Otherwise, you will receive a constraint failure at runtime when the vec is pushed to.

### get

```rust
pub fn get(self, index: u64) -> T {
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
pub fn get_unchecked(self, index: u64) -> T {
```

Retrieves an element from the vector at the given index, starting from zero, without
performing a bounds check.

Since this function does not perform a bounds check on length before accessing the element,
it is unsafe! Use at your own risk!

Example:

```rust title="get_unchecked_example" showLineNumbers 
fn sum_of_first_three<let N: u32>(v: BoundedVec<u32, N>) -> u32 {
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


### set

```rust
pub fn set(&mut self: Self, index: u64, value: T) {
```

Writes an element to the vector at the given index, starting from zero.

If the given index is equal to or greater than the length of the vector, this will issue a constraint failure.

Example:

```rust
fn foo<N>(v: BoundedVec<u32, N>) {
    let first = v.get(0);
    assert(first != 42);
    v.set(0, 42);
    let new_first = v.get(0);
    assert(new_first == 42);
}
```

### set_unchecked

```rust
pub fn set_unchecked(&mut self: Self, index: u64, value: T) -> T {
```

Writes an element to the vector at the given index, starting from zero, without performing a bounds check.

Since this function does not perform a bounds check on length before accessing the element, it is unsafe! Use at your own risk!

Example:

```rust title="set_unchecked_example" showLineNumbers 
fn set_unchecked_example() {
    let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
    vec.extend_from_array([1, 2]);

    // Here we're safely writing within the valid range of `vec`
    // `vec` now has the value [42, 2]
    vec.set_unchecked(0, 42);

    // We can then safely read this value back out of `vec`.
    // Notice that we use the checked version of `get` which would prevent reading unsafe values.
    assert_eq(vec.get(0), 42);

    // We've now written past the end of `vec`.
    // As this index is still within the maximum potential length of `v`,
    // it won't cause a constraint failure.
    vec.set_unchecked(2, 42);
    println(vec);

    // This will write past the end of the maximum potential length of `vec`,
    // it will then trigger a constraint failure.
    vec.set_unchecked(5, 42);
    println(vec);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L67-L91" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L67-L91</a></sub></sup>



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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L95-L103" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L95-L103</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L108-L120" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L108-L120</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L125-L140" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L125-L140</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L145-L151" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L145-L151</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L156-L163" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L156-L163</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L168-L175" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L168-L175</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L180-L189" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L180-L189</a></sub></sup>


### from_array

```rust
pub fn from_array<Len>(array: [T; Len]) -> Self
```

Creates a new vector, populating it with values derived from an array input. 
The maximum length of the vector is determined based on the type signature.

Example:
```rust
let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from_array([1, 2, 3])
```

### from_parts

```rust
pub fn from_parts(mut array: [T; MaxLen], len: u32) -> Self
```

Creates a new BoundedVec from the given array and length.
The given length must be less than or equal to the length of the array.

This function will zero out any elements at or past index `len` of `array`.
This incurs an extra runtime cost of O(MaxLen). If you are sure your array is
zeroed after that index, you can use `from_parts_unchecked` to remove the extra loop.

Example:

```rust title="from-parts" showLineNumbers 
let vec: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 0], 3);
            assert_eq(vec.len(), 3);

            // Any elements past the given length are zeroed out, so these
            // two BoundedVecs will be completely equal
            let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 1], 3);
            let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 2], 3);
            assert_eq(vec1, vec2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/bounded_vec.nr#L663-L672" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/bounded_vec.nr#L663-L672</a></sub></sup>


### from_parts_unchecked

```rust
pub fn from_parts_unchecked(array: [T; MaxLen], len: u32) -> Self
```

Creates a new BoundedVec from the given array and length.
The given length must be less than or equal to the length of the array.

This function is unsafe because it expects all elements past the `len` index
of `array` to be zeroed, but does not check for this internally. Use `from_parts`
for a safe version of this function which does zero out any indices past the
given length. Invalidating this assumption can notably cause `BoundedVec::eq`
to give incorrect results since it will check even elements past `len`.

Example:

```rust title="from-parts-unchecked" showLineNumbers 
let vec: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 0], 3);
            assert_eq(vec.len(), 3);

            // invalid use!
            let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 1], 3);
            let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 2], 3);

            // both vecs have length 3 so we'd expect them to be equal, but this
            // fails because elements past the length are still checked in eq
            assert(vec1 != vec2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/bounded_vec.nr#L677-L688" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/bounded_vec.nr#L677-L688</a></sub></sup>


### map

```rust
pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> BoundedVec<U, MaxLen>
```

Creates a new vector of equal size by calling a closure on each element in this vector.  

Example:

```rust title="bounded-vec-map-example" showLineNumbers 
let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
            let result = vec.map(|value| value * 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/bounded_vec.nr#L551-L554" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/bounded_vec.nr#L551-L554</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/bounded_vec/src/main.nr#L256-L262" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/bounded_vec/src/main.nr#L256-L262</a></sub></sup>

