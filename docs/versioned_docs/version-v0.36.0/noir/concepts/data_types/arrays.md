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
sidebar_position: 4
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

Like in Rust, arrays in Noir are a fixed size. However, if you wish to convert an array to a [slice](./slices.mdx), you can just call `as_slice` on your array:

```rust
let array: [Field; 32] = [0; 32];
let sl = array.as_slice()
```

You can define multidimensional arrays:

```rust
let array : [[Field; 2]; 2];
let element = array[0][0];
```

However, multidimensional slices are not supported. For example, the following code will error at compile time:

```rust
let slice : [[Field]] = &[];
```

## Types

You can create arrays of primitive types or structs. There is not yet support for nested arrays
(arrays of arrays) or arrays of structs that contain arrays.

## Methods

For convenience, the STD provides some ready-to-use, common methods for arrays.
Each of these functions are located within the generic impl `impl<T, N> [T; N] {`.
So anywhere `self` appears, it refers to the variable `self: [T; N]`.

### len

Returns the length of an array

```rust
fn len(self) -> Field
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
fn sort(self) -> [T; N]
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

Sorts the array with a custom comparison function. The ordering function must return true if the first argument should be sorted to be before the second argument or is equal to the second argument.

Using this method with an operator like `<` that does not return `true` for equal values will result in an assertion failure for arrays with equal elements.

```rust
fn sort_via(self, ordering: fn(T, T) -> bool) -> [T; N]
```

example

```rust
fn main() {
    let arr = [42, 32]
    let sorted_ascending = arr.sort_via(|a, b| a <= b);
    assert(sorted_ascending == [32, 42]); // verifies

    let sorted_descending = arr.sort_via(|a, b| a >= b);
    assert(sorted_descending == [32, 42]); // does not verify
}
```

### map

Applies a function to each element of the array, returning a new array containing the mapped elements.

```rust
fn map<U>(self, f: fn(T) -> U) -> [U; N]
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
fn fold<U>(self, mut accumulator: U, f: fn(U, T) -> U) -> U
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

Same as fold, but uses the first element as the starting element.

```rust
fn reduce(self, f: fn(T, T) -> T) -> T
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
fn all(self, predicate: fn(T) -> bool) -> bool
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
fn any(self, predicate: fn(T) -> bool) -> bool
```

example:

```rust
fn main() {
    let arr = [2, 2, 2, 2, 5];
    let any = arr.any(|a| a == 5);
    assert(any);
}
```

### as_str_unchecked

Converts a byte array of type `[u8; N]` to a string. Note that this performs no UTF-8 validation -
the given array is interpreted as-is as a string.

```rust
impl<let N: u32> [u8; N] {
    pub fn as_str_unchecked(self) -> str<N>
}
```

example:

```rust
fn main() {
    let hi = [104, 105].as_str_unchecked();
    assert_eq(hi, "hi");
}
```
