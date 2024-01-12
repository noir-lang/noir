---
title: Arrays
description:
  Dive into the Array data type in Noir. Grasp its methods, practical examples, and best practices for efficiently using Arrays in your Noir code.
keywords:
  [
    noir,
    array type,
    methods,
    examples,
    indexing,
  ]
---

An array is one way of grouping together values into one compound type. Array types can be inferred
or explicitly specified via the syntax `[<Type>; <Size>]`:

```rust
fn main(x : Field, y : Field) {
    let my_arr = [x, y];
    let your_arr: [Field; 2] = [x, y];
}
```

Here, both `my_arr` and `your_arr` are instantiated as an array containing two `Field` elements.

Array elements can be accessed using indexing:

```rust
fn main() {
    let a = [1, 2, 3, 4, 5];

    let first = a[0];
    let second = a[1];
}
```

All elements in an array must be of the same type (i.e. homogeneous). That is, an array cannot group
a `Field` value and a `u8` value together for example.

You can write mutable arrays, like:

```rust
fn main() {
    let mut arr = [1, 2, 3, 4, 5];
    assert(arr[0] == 1);

    arr[0] = 42;
    assert(arr[0] == 42);
}
```

You can instantiate a new array of a fixed size with the same value repeated for each element. The following example instantiates an array of length 32 where each element is of type Field and has the value 0.

```rust
let array: [Field; 32] = [0; 32];
```

Like in Rust, arrays in Noir are a fixed size. However, if you wish to convert an array to a [slice](./slices), you can just call `as_slice` on your array:

```rust
let array: [Field; 32] = [0; 32];
let sl = array.as_slice()
```

You can define multidimensional arrays:

```rust
let array : [[Field; 2]; 2];
let element = array[0][0];
```

## Types

You can create arrays of primitive types or structs. There is not yet support for nested arrays
(arrays of arrays) or arrays of structs that contain arrays.

## Methods

For convenience, the STD provides some ready-to-use, common methods for arrays:

### len

Returns the length of an array

```rust
fn len<T, N>(_array: [T; N]) -> comptime Field
```

example

```rust
fn main() {
    let array = [42, 42];
    assert(array.len() == 2);
}
```

### sort

Returns a new sorted array. The original array remains untouched. Notice that this function will
only work for arrays of fields or integers, not for any arbitrary type. This is because the sorting
logic it uses internally is optimized specifically for these values. If you need a sort function to
sort any type, you should use the function `sort_via` described below.

```rust
fn sort<T, N>(_array: [T; N]) -> [T; N]
```

example

```rust
fn main() {
    let arr = [42, 32];
    let sorted = arr.sort();
    assert(sorted == [32, 42]);
}
```

### sort_via

Sorts the array with a custom comparison function

```rust
fn sort_via<T, N>(mut a: [T; N], ordering: fn(T, T) -> bool) -> [T; N]
```

example

```rust
fn main() {
    let arr = [42, 32]
    let sorted_ascending = arr.sort_via(|a, b| a < b);
    assert(sorted_ascending == [32, 42]); // verifies

    let sorted_descending = arr.sort_via(|a, b| a > b);
    assert(sorted_descending == [32, 42]); // does not verify
}
```

### map

Applies a function to each element of the array, returning a new array containing the mapped elements.

```rust
fn map<U>(f: fn(T) -> U) -> [U; N]
```

example

```rust
let a = [1, 2, 3];
let b = a.map(|a| a * 2); // b is now [2, 4, 6]
```

### fold

Applies a function to each element of the array, returning the final accumulated value. The first
parameter is the initial value.

```rust
fn fold<U>(mut accumulator: U, f: fn(U, T) -> U) -> U
```

This is a left fold, so the given function will be applied to the accumulator and first element of
the array, then the second, and so on. For a given call the expected result would be equivalent to:

```rust
let a1 = [1];
let a2 = [1, 2];
let a3 = [1, 2, 3];

let f = |a, b| a - b;
a1.fold(10, f)  //=> f(10, 1)
a2.fold(10, f)  //=> f(f(10, 1), 2)
a3.fold(10, f)  //=> f(f(f(10, 1), 2), 3)
```

example:

```rust

fn main() {
    let arr = [2, 2, 2, 2, 2];
    let folded = arr.fold(0, |a, b| a + b);
    assert(folded == 10);
}

```

### reduce

Same as fold, but uses the first element as starting element.

```rust
fn reduce<T, N>(f: fn(T, T) -> T) -> T
```

example:

```rust
fn main() {
    let arr = [2, 2, 2, 2, 2];
    let reduced = arr.reduce(|a, b| a + b);
    assert(reduced == 10);
}
```

### all

Returns true if all the elements satisfy the given predicate

```rust
fn all<T, N>(predicate: fn(T) -> bool) -> bool
```

example:

```rust
fn main() {
    let arr = [2, 2, 2, 2, 2];
    let all = arr.all(|a| a == 2);
    assert(all);
}
```

### any

Returns true if any of the elements satisfy the given predicate

```rust
fn any<T, N>(predicate: fn(T) -> bool) -> bool
```

example:

```rust
fn main() {
    let arr = [2, 2, 2, 2, 5];
    let any = arr.any(|a| a == 5);
    assert(any);
}

```
