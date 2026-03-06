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
fn main(x : u64, y : u64) {
    let my_arr = [x, y];
    let your_arr: [u64; 2] = [x, y];
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

Like in Rust, arrays in Noir are a fixed size. However, if you wish to convert an array to a [vector](./vectors.mdx), you can just call `as_vector` on your array:

```rust
let array: [Field; 32] = [0; 32];
let sl = array.as_vector()
```

You can define multidimensional arrays:

```rust
let array : [[Field; 2]; 2];
let element = array[0][0];
```

However, multidimensional vectors are not supported. For example, the following code will error at compile time:

```rust
let vector : [[Field]] = @[];
```

## Dynamic Indexing

Using constant indices of arrays will often be more efficient at runtime in constrained code.
Indexing an array with non-constant indices (indices derived from the inputs to the program, or returned from unconstrained functions) is also
called "dynamic indexing" and incurs a slight runtime cost:

```rust
fn main(x: u32) {
    let array = [1, 2, 3, 4];

    // This is a constant index, after inlining the compiler sees that this
    // will always be `array[2]`
    let _a = array[double(1)];

    // This is a non-constant index, there is no way to know which u32 value
    // will be used as an index here
    let _b = array[double(x)];
}

fn double(y: u32) -> u32 {
    y * 2
}
```

There is another restriction with dynamic indices: they cannot be used on arrays with
elements which contain a reference type:

```rust
fn main(x: u32) {
    let array = [&mut 1, &mut 2, &mut 3, &mut 4];

    // error! Only constant indices may be used here since `array` contains references internally!
    let _c = array[x];
}
```

## Indexing with Field Elements
Working with the native Field type can help producing more optimized programs (if you know what you are doing!), by avoiding overflow checks in general and in particular for array index, u32 arithmetic.
However Noir type system will require your array index to be `u32`, so if you computed an array index using the Field type, you will have to convert it into a `u32`. This operation is usually costly because the 'unbounded' Field element needs to be reduced modulo `2^32`. However we can benefit from the array out-of-bound checks in order to avoid this costly operation.
One way to do it is the following:
1. use `assert_max_bit_size::<32>();` and `as u32` in order to convert a Field into a u32 using only one range-check, if you know that the Field is indeed 32 bits (or less).
2. Index the array with the resulting u32: `array[x as u32]`. This will remove the range-check from the previous step.

```rust
fn foo(x: Field, array: [Field; 10]) -> Field {
    // x is used to index `array`, so it must fit into 32 bits
    x.assert_max_bit_size::<32>();
    // assert_max_bit_size::<32>() makes the u32 conversion: `x as u32`, free.
    // Accessing the array also implies an out-of-bound check, so it makes the range-check `x.assert_max_bit_size::<32>();`
    // redundant. It will be removed in a later stage.
    array[x as u32]
}
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

### mapi

Applies a function to each element of the array, along with its index in the
array, returning a new array containing the mapped elements.

```rust
fn mapi<U, Env>(self, f: fn[Env](u32, T) -> U) -> [U; N]
```

example

```rust
let a = [1, 2, 3];
let b = a.mapi(|i, a| i + a * 2); // b is now [2, 5, 8]
```

### for_each

Applies a function to each element of the array.

```rust
fn for_each<Env>(self, f: fn[Env](T) -> ())
```

example

```rust
let a = [1, 2, 3];
a.for_each(|x| {
    println(f"{x}");
});
// prints:
// 1
// 2
// 3
```

### for_eachi

Applies a function to each element of the array, along with its index in the
array.

```rust
fn for_eachi<Env>(self, f: fn[Env](u32, T) -> ())
```

example

```rust
let a = [1, 2, 3];
a.for_eachi(|i, x| {
    println(f"{i}, {x}");
});
// prints:
// 0, 1
// 1, 2
// 2, 3
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

Requires `self` to be non-empty.

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

### concat

Concatenates this array with another array.

```rust
fn concat<let M: u32>(self, array2: [T; M]) -> [T; N + M]
```

```rust
fn main() {
    let arr1 = [1, 2, 3, 4];
    let arr2 = [6, 7, 8, 9, 10, 11];
    let concatenated_arr = arr1.concat(arr2);
    assert(concatenated_arr == [1, 2, 3, 4, 6, 7, 8, 9, 10, 11]);
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
