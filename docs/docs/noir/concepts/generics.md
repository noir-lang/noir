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

## Numeric Generics

If we want to be generic over array lengths (which are type-level integers), we can use numeric
generics. Using these looks similar to using regular generics, but introducing them into scope
requires declaring them with `let MyGenericName: IntegerType`. This can be done anywhere a normal
generic is declared. Instead of types, these generics resolve to integers at compile-time.
Here's an example of a struct that is generic over the size of the array it contains internally:

```rust
struct BigInt<let N: u32> {
    limbs: [u32; N],
}

impl<let N: u32> BigInt<N> {
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

## Calling functions on generic parameters

Since a generic type `T` can represent any type, how can we call functions on the underlying type?
In other words, how can we go from "any type `T`" to "any type `T` that has certain methods available?"

This is what [traits](../concepts/traits.md) are for in Noir. Here's an example of a function generic over
any type `T` that implements the `Eq` trait for equality:

```rust
fn first_element_is_equal<T, let N: u32>(array1: [T; N], array2: [T; N]) -> bool 
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

You can find more details on traits and trait implementations on the [traits page](../concepts/traits.md).

## Manually Specifying Generics with the Turbofish Operator

There are times when the compiler cannot reasonably infer what type should be used for a generic, or when the developer themselves may want to manually distinguish generic type parameters. This is where the `::<>` turbofish operator comes into play.

The `::<>` operator can follow a variable or path and can be used to manually specify generic arguments within the angle brackets.
The name "turbofish" comes from that `::<>` looks like a little fish.

Examples:
```rust
fn main() {
    let mut slice = [];
    slice = slice.push_back(1);
    slice = slice.push_back(2);
    // Without turbofish a type annotation would be needed on the left hand side
    let array = slice.as_array::<2>();
}
```


```rust
trait MyTrait {
    fn ten() -> Self;
}

impl MyTrait for Field {
    fn ten() -> Self { 10 }
}

struct Foo<T> {
    inner: T
}
        
impl<T> Foo<T> {
    fn generic_method<U>(_self: Self) -> U where U: MyTrait {
        U::ten()
    }
}
        
fn example() {
    let foo: Foo<Field> = Foo { inner: 1 };
    // Using a type other than `Field` here (e.g. u32) would fail as 
    // there is no matching impl for `u32: MyTrait`. 
    //
    // Substituting the `10` on the left hand side of this assert
    // with `10 as u32` would also fail with a type mismatch as we 
    // are expecting a `Field` from the right hand side.
    assert(10 as u32 == foo.generic_method::<Field>());
}
```

## Arithmetic Generics

In addition to numeric generics, Noir also allows a limited form of arithmetic on generics.
When you have a numeric generic such as `N`, you can use the following operators on it in a
type position: `+`, `-`, `*`, `/`, and `%`.

Note that type checking arithmetic generics is a best effort guess from the compiler and there
are many cases of types that are equal that the compiler may not see as such. For example,
we know that `T * (N + M)` should be equal to `T*N + T*M` but the compiler does not currently
apply the distributive law and thus sees these as different types.

Even with this limitation though, the compiler can handle common cases decently well:

```rust
trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}

impl Serialize<1> for Field {
    fn serialize(self) -> [Field; 1] {
        [self]
    }
}

impl<T, let N: u32, let M: u32> Serialize<N * M> for [T; N]
    where T: Serialize<M> { .. }

impl<T, U, let N: u32, let M: u32> Serialize<N + M> for (T, U)
    where T: Serialize<N>, U: Serialize<M> { .. }

fn main() {
    let data = (1, [2, 3, 4]);
    assert_eq(data.serialize().len(), 4);
}
```

Note that if there is any over or underflow the types will fail to unify:

#include_code underflow-example test_programs/compile_failure/arithmetic_generics_underflow/src/main.nr rust

This also applies if there is underflow in an intermediate calculation:

#include_code intermediate-underflow-example test_programs/compile_failure/arithmetic_generics_intermediate_underflow/src/main.nr rust
