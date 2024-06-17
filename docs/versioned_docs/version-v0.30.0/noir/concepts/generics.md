---
title: Generics
description: Learn how to use Generics in Noir
keywords: [Noir, Rust, generics, functions, structs]
sidebar_position: 7
---

Generics allow you to use the same functions with multiple different concrete data types. You can
read more about the concept of generics in the Rust documentation
[here](https://doc.rust-lang.org/book/ch10-01-syntax.html).

Here is a trivial example showing the identity function that supports any type. In Rust, it is
common to refer to the most general type as `T`. We follow the same convention in Noir.

```rust
fn id<T>(x: T) -> T  {
    x
}
```

## In Structs

Generics are useful for specifying types in structs. For example, we can specify that a field in a
struct will be of a certain generic type. In this case `value` is of type `T`.

```rust
struct RepeatedValue<T> {
    value: T,
    count: Field,
}

impl<T> RepeatedValue<T> {
    fn print(self) {
        for _i in 0 .. self.count {
            println(self.value);
        }
    }
}

fn main() {
    let repeated = RepeatedValue { value: "Hello!", count: 2 };
    repeated.print();
}
```

The `print` function will print `Hello!` an arbitrary number of times, twice in this case.

If we want to be generic over array lengths (which are type-level integers), we can use numeric
generics. Using these looks just like using regular generics, but these generics can resolve to
integers at compile-time, rather than resolving to types. Here's an example of a struct that is
generic over the size of the array it contains internally:

```rust
struct BigInt<N> {
    limbs: [u32; N],
}

impl<N> BigInt<N> {
    // `N` is in scope of all methods in the impl
    fn first(first: BigInt<N>, second: BigInt<N>) -> Self {
        assert(first.limbs != second.limbs);
        first

    fn second(first: BigInt<N>, second: Self) -> Self {
        assert(first.limbs != second.limbs);
        second
    }
}
```

## Calling functions on generic parameters

Since a generic type `T` can represent any type, how can we call functions on the underlying type?
In other words, how can we go from "any type `T`" to "any type `T` that has certain methods available?"

This is what [traits](../concepts/traits) are for in Noir. Here's an example of a function generic over
any type `T` that implements the `Eq` trait for equality:

```rust
fn first_element_is_equal<T, N>(array1: [T; N], array2: [T; N]) -> bool 
    where T: Eq
{
    if (array1.len() == 0) | (array2.len() == 0) {
        true
    } else {
        array1[0] == array2[0]
    }
}

fn main() {
    assert(first_element_is_equal([1, 2, 3], [1, 5, 6]));

    // We can use first_element_is_equal for arrays of any type
    // as long as we have an Eq impl for the types we pass in
    let array = [MyStruct::new(), MyStruct::new()];
    assert(array_eq(array, array, MyStruct::eq));
}

impl Eq for MyStruct {
    fn eq(self, other: MyStruct) -> bool {
        self.foo == other.foo
    }
}
```

You can find more details on traits and trait implementations on the [traits page](../concepts/traits).
