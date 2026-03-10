---
title: Bounded Vectors
description: Growable vectors with a fixed maximum length; safer and more efficient than vectors, with rich methods for access and mutation.
keywords: [noir, vector, bounded vector, vector]
sidebar_position: 1
---

A `BoundedVec<T, MaxLen>` is a growable storage similar to a `Vec<T>` except that it
is bounded with a maximum possible length. Unlike `Vec`, `BoundedVec` is not implemented
via vectors and thus is not subject to the same restrictions vectors are (notably, nested
vectors - and thus nested vectors as well - are disallowed).

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

#include_code new_example test_programs/noir_test_success/bounded_vec/src/main.nr rust

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

#include_code get_unchecked_example test_programs/noir_test_success/bounded_vec/src/main.nr rust

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

#include_code set_unchecked_example test_programs/noir_test_success/bounded_vec/src/main.nr rust


### push

```rust
pub fn push(&mut self, elem: T) {
```

Pushes an element to the end of the vector. This increases the length
of the vector by one.

Panics if the new length of the vector will be greater than the max length.

Example:

#include_code bounded-vec-push-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### pop

```rust
pub fn pop(&mut self) -> T
```

Pops the element at the end of the vector. This will decrease the length
of the vector by one.

Panics if the vector is empty.

Example:

#include_code bounded-vec-pop-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### len

```rust
pub fn len(self) -> u64 {
```

Returns the current length of this vector

Example:

#include_code bounded-vec-len-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### max_len

```rust
pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u64 {
```

Returns the maximum length of this vector. This is always
equal to the `MaxLen` parameter this vector was initialized with.

Example:

#include_code bounded-vec-max-len-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### set_len

```rust
pub fn set_len(&mut self, len: u32) {
```

Sets the length of the BoundedVec manually. This is useful to update the length after
updating the content of the vector using `set_unchecked`.

Example:

#include_code bounded-vec-set-len-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### storage

```rust
pub fn storage(self) -> [T; MaxLen] {
```

Returns the internal array within this vector.
Since arrays in Noir are immutable, mutating the returned storage array will not mutate
the storage held internally by this vector.

Note that uninitialized elements may be zeroed out!

Example:

#include_code bounded-vec-storage-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### extend_from_array

```rust
pub fn extend_from_array<Len>(&mut self, array: [T; Len])
```

Pushes each element from the given array to this vector.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

#include_code bounded-vec-extend-from-array-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

### extend_from_bounded_vec

```rust
pub fn extend_from_bounded_vec<Len>(&mut self, vec: BoundedVec<T, Len>)
```

Pushes each element from the other vector to this vector. The length of
the other vector is left unchanged.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

#include_code bounded-vec-extend-from-bounded-vec-example test_programs/noir_test_success/bounded_vec/src/main.nr rust

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

#include_code from-parts noir_stdlib/src/collections/bounded_vec.nr rust

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

#include_code from-parts-unchecked noir_stdlib/src/collections/bounded_vec.nr rust

### map

```rust
pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> BoundedVec<U, MaxLen>
```

Creates a new vector of equal size by calling a closure on each element in this vector.

Example:

#include_code bounded-vec-map-example noir_stdlib/src/collections/bounded_vec.nr rust

### mapi

```rust
pub fn mapi<U, Env>(self, f: fn[Env](u32, T) -> U) -> BoundedVec<U, MaxLen>
```

Creates a new vector of equal size by calling a closure on each element in this
vector, along with its index in the vector.

Example:

#include_code bounded-vec-mapi-example noir_stdlib/src/collections/bounded_vec.nr rust

### for_each

```rust
pub fn for_each<Env>(self, f: fn[Env](T) -> ())
```

Calls a closure on each element in this vector.

Example:

#include_code bounded-vec-for-each-example noir_stdlib/src/collections/bounded_vec.nr rust

### for_eachi

```rust
pub fn for_eachi<Env>(self, f: fn[Env](u32, T) -> ())
```

Calls a closure on each element in this vector, along with its index in the vector.

Example:

#include_code bounded-vec-for-eachi-example noir_stdlib/src/collections/bounded_vec.nr rust

### any

```rust
pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

Returns true if the given predicate returns true for any element
in this vector.

Example:

#include_code bounded-vec-any-example test_programs/noir_test_success/bounded_vec/src/main.nr rust
