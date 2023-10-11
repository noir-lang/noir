---
title: Slice Methods
description:
  Learn about the commonly used methods available for slices in Noir, including push_back, len, srt, map, fold, reduce, all, and any.
keywords: [rust, slice, methods, push_back, len, sort, fold, reduce, all, any]
---

For convenience, the STD provides some ready-to-use, common methods for slices:

## push_back

Pushes a new element to the end of the slice, returning a new slice with a length one greater than the original unmodified slice.

```rust
fn push_back<T>(_self: [T], _elem: T) -> [T]
```

example:

```rust
fn main() -> pub Field {
    let mut slice: [Field] = [0; 2];

    let mut new_slice = slice.push_back(6);
    new_slice.len()
}
```

View the corresponding test file [here][test-file].

## push_front

Returns a new array with the specified element inserted at index 0. The existing elements indexes are incremented by 1.

```rust
fn push_front(_self: Self, _elem: T) -> Self
```

Example:

```rust
let mut new_slice: [Field] = [];
new_slice = new_slice.push_front(20);
assert(new_slice[0] == 20); // returns true
```

View the corresponding test file [here][test-file].

## pop_front

Returns a tuple of two items, the first element of the array and the rest of the array.

```rust
fn pop_front(_self: Self) -> (T, Self)
```

Example:

```rust
let (first_elem, rest_of_slice) = slice.pop_front();
```

View the corresponding test file [here][test-file].

## pop_back

Returns a tuple of two items, the beginning of the array with the last element omitted and the last element.

```rust
fn pop_back(_self: Self) -> (Self, T)
```

Example:

```rust
let (popped_slice, last_elem) = slice.pop_back();
```

View the corresponding test file [here][test-file].

## insert

Inserts an element at a specified index and shifts all following elements by 1.

```rust
fn insert(_self: Self, _index: Field, _elem: T) -> Self
```

Example:

```rust
 new_slice = rest_of_slice.insert(2, 100);
assert(new_slice[2] == 100);
```

View the corresponding test file [here][test-file].

## remove

Remove an element at a specified index, shifting all elements after it to the left, returning the altered slice and the removed element.

```rust
fn remove(_self: Self, _index: Field) -> (Self, T)
```

Example:

```rust
let (remove_slice, removed_elem) = slice.remove(3);
```

View the corresponding test file [here]([test-file].

## len

Returns the length of a slice

```rust
fn len<T>(_slice: [T]) -> comptime Field
```

Example:

```rust
fn main() {
    let slic = [42, 42]
    assert(slic.len() == 2);
}
```

## sort

Returns a new sorted slice. The original slice remains untouched. Notice that this function will
only work for slices of fields or integers, not for any arbitrary type. This is because the sorting
logic the function uses internally is optimized specifically for these values. If you need a sort function to
sort any type, you should use the function `sort_via` described below.

```rust
fn sort<T>(_slice: [T]) -> [T]
```

Example:

```rust
fn main() {
    let slic = [42, 32]
    let sorted = slic.sort();
    assert(sorted == [32, 42]);
}
```

## sort_via

Sorts the slice with a custom comparison function

```rust
fn sort_via<T>(mut a: [T], ordering: fn(T, T) -> bool) -> [T]
```

Example:

```rust
fn main() {
    let slic = [42, 32]
    let sorted_ascending = slic.sort_via(|a, b| a < b);
    assert(sorted_ascending == [32, 42]); // verifies

    let sorted_descending = slic.sort_via(|a, b| a > b);
    assert(sorted_descending == [32, 42]); // does not verify
}
```

## map

Applies a function to each element of the slice, returning a new slice containing the mapped elements.

```rust
fn map<U>(f: fn(T) -> U) -> [U]
```

Example:

```rust
let a = [1, 2, 3];
let b = a.map(|a| a * 2) // b is now [2, 4, 6]
```

## fold

Applies a function to each element of the slice, returning the final accumulated value. The first
parameter is the initial value.

```rust
fn fold<U>(mut accumulator: U, f: fn(U, T) -> U) -> U
```

This is a left fold, so the given function will be applied to the accumulator and first element of
the slice, then the second, and so on. For a given call the expected result would be equivalent to:

```rust
let a1 = [1];
let a2 = [1, 2];
let a3 = [1, 2, 3];

let f = |a, b| a - b;
a1.fold(10, f)  //=> f(10, 1)
a2.fold(10, f)  //=> f(f(10, 1), 2)
a3.fold(10, f)  //=> f(f(f(10, 1), 2), 3)
```

Example:

```rust

fn main() {
    let slic = [2,2,2,2,2]
    let folded = slic.fold(0, |a, b| a + b);
    assert(folded == 10);
}

```

## reduce

Same as fold, but uses the first element as starting element.

```rust
fn reduce<T>(f: fn(T, T) -> T) -> T
```

Example:

```rust
fn main() {
    let slic = [2,2,2,2,2]
    let reduced = slic.reduce(|a, b| a + b);
    assert(reduced == 10);
}
```

## all

Returns true if all the elements satisfy the given predicate

```rust
fn all<T>(predicate: fn(T) -> bool) -> bool
```

Example:

```rust
fn main() {
    let slic = [2,2,2,2,2]
    let all = slic.all(|a| a == 2);
    assert(all);
}
```

## any

Returns true if any of the elements satisfy the given predicate

```rust
fn any<T>(predicate: fn(T) -> bool) -> bool
```

Example:

```rust
fn main() {
    let slic = [2,2,2,2,5]
    let any = slic.any(|a| a == 5);
    assert(any);
}

```

[test-file]: https://github.com/noir-lang/noir/blob/f387ec1475129732f72ba294877efdf6857135ac/crates/nargo_cli/tests/test_data_ssa_refactor/slices/src/main.nr

