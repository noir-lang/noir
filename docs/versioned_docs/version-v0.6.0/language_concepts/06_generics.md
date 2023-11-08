---
title: Generics
description:
  Learn how to use Generics in Noir
keywords: [Noir, Rust, generics, functions, structs]
---

# Generics

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
    count: comptime Field,
}

impl<T> RepeatedValue<T> {
    fn new(value: T) -> Self {
        Self { value, count: 1 }
    }

    fn increment(mut repeated: Self) -> Self {
        repeated.count += 1;
        repeated
    }

    fn print(self) {
        for _i in 0 .. self.count {
            dep::std::println(self.value);
        }
    }
}

fn main() {
    let mut repeated = RepeatedValue::new("Hello!");
    repeated = repeated.increment();
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
        constrain first.limbs != second.limbs;
        first

    fn second(first: BigInt<N>, second: Self) -> Self {
        constrain first.limbs != second.limbs;
        second
    }
}
```

## Calling functions on generic parameters

Unlike Rust, Noir does not have traits, so how can one translate the equivalent of a trait bound in
Rust into Noir? That is, how can we write a function that is generic over some type `T`, while also
requiring there is a function like `eq: fn(T, T) -> bool` that works on the type?

The answer is that we can translate this by passing in the function manually. Here's an example of
implementing array equality in Noir:

```rust
fn array_eq<T, N>(array1: [T; N], array2: [T; N], elem_eq: fn(T, T) -> bool) -> bool {
    if array1.len() != array2.len() {
        false
    } else {
        let mut result = true;
        for i in 0 .. array1.len() {
            result &= elem_eq(array1[i], array2[i]);
        }
        result
    }
}

fn main() {
    constrain array_eq([1, 2, 3], [1, 2, 3], |a, b| a == b);

    // We can use array_eq even for arrays of structs, as long as we have
    // an equality function for these structs we can pass in
    let array = [MyStruct::new(), MyStruct::new()];
    constrain array_eq(array, array, MyStruct::eq);
}
```

You can see an example of generics in the tests
[here](https://github.com/noir-lang/noir/blob/master/crates/nargo_cli/tests/test_data/generics/src/main.nr).
