---
title: Data Types
description:
  Get a clear understanding of the two categories of Noir data types - primitive types and compound
  types. Learn about their characteristics, differences, and how to use them in your Noir
  programming.
keywords:
  [
    noir,
    data types,
    primitive types,
    compound types,
    private types,
    public types,
    field type,
    integer types,
    boolean type,
    array type,
    tuple type,
    struct type,
  ]
---

Every value in Noir has a type, which determines which operations are valid for it.

All values in Noir are fundamentally composed of `Field` elements. For a more approachable
developing experience, abstractions are added on top to introduce different data types in Noir.

Noir has two category of data types: primitive types (e.g. `Field`, integers, `bool`) and compound
types that group primitive types (e.g. arrays, tuples, structs). Each value can either be private or
public.

## Private & Public Types

A **private value** is known only to the Prover, while a **public value** is known by both the
Prover and Verifier. Mark values as `private` when the value should only be known to the prover. All
primitive types (including individual fields of compound types) in Noir are private by default, and
can be marked public when certain values are intended to be revealed to the Verifier.

> **Note:** For public values defined in Noir programs paired with smart contract verifiers, once
> the proofs are verified on-chain the values can be considered known to everyone that has access to
> that blockchain.

Public data types are treated no differently to private types apart from the fact that their values
will be revealed in proofs generated. Simply changing the value of a public type will not change the
circuit (where the same goes for changing values of private types as well).

_Private values_ are also referred to as _witnesses_ sometimes.

> **Note:** The terms private and public when applied to a type (e.g. `pub Field`) have a different
> meaning than when applied to a function (e.g. `pub fn foo() {}`).
>
> The former is a visibility modifier for the Prover to interpret if a value should be made known to
> the Verifier, while the latter is a visibility modifier for the compiler to interpret if a
> function should be made accessible to external Noir programs like in other languages.

### pub Modifier

All data types in Noir are private by default. Types are explicitly declared as public using the
`pub` modifier:

```rust
fn main(x : Field, y : pub Field) -> pub Field {
    x + y
}
```

In this example, `x` is **private** while `y` and `x + y` (the return value) are **public**. Note
that visibility is handled **per variable**, so it is perfectly valid to have one input that is
private and another that is public.

> **Note:** Public types can only be declared through parameters on `main`.

## Primitive Types

A primitive type represents a single value. They can be private or public.

### Fields

The field type corresponds to the native field type of the proving backend.

The size of a Noir field depends on the elliptic curve's finite field for the proving backend
adopted. For example, a field would be a 254-bit integer when paired with the default backend that
spans the Grumpkin curve.

Fields support integer arithmetic and are often used as the default numeric type in Noir:

```rust
fn main(x : Field, y : Field)  {
    let z = x + y;
}
```

`x`, `y` and `z` are all private fields in this example. Using the `let` keyword we defined a new
private value `z` constrained to be equal to `x + y`.

If proving efficiency is of priority, fields should be used as a default for solving problems.
Smaller integer types (e.g. `u64`) incur extra range constraints.

### Integers

An integer type is a range constrained field type. The Noir frontend currently supports unsigned,
arbitrary-sized integer types.

An integer type is specified first with the letter `u`, indicating its unsigned nature, followed by
its length in bits (e.g. `32`). For example, a `u32` variable can store a value in the range of
$\\([0,2^{32}-1]\\)$:

```rust
fn main(x : Field, y : u32) {
    let z = x as u32 + y;
}
```

`x`, `y` and `z` are all private values in this example. However, `x` is a field while `y` and `z`
are unsigned 32-bit integers. If `y` or `z` exceeds the range $\\([0,2^{32}-1]\\)$, proofs created
will be rejected by the verifier.

> **Note:** The default backend supports both even (e.g. `u16`, `u48`) and odd (e.g. `u5`, `u3`)
> sized integer types.

### Booleans

The `bool` type in Noir has two possible values: `true` and `false`:

```rust
fn main() {
    let t = true;
    let f: bool = false;
}
```

> **Note:** When returning a boolean value, it will show up as a value of 1 for `true` and 0 for
> `false` in _Verifier.toml_.

The boolean type is most commonly used in conditionals like `if` expressions and `assert`
statements. More about conditionals is covered in the [Control Flow](./control_flow) and
[Assert Function](./assert) sections.

### Strings

The string type is a fixed length value defined with `str<N>`.

You can use strings in `assert()` functions or print them with
`std::println()`.

```rust
fn main(message : pub str<11>, hex_as_string : str<4>) {
    std::println(message);
    assert(message == "hello world");
    assert(hex_as_string == "0x41");
}
```

## Compound Types

A compound type groups together multiple values into one type. Elements within a compound type can
be private or public.

### Arrays

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

#### Types

You can create arrays of primitive types or structs. There is not yet support for nested arrays
(arrays of arrays) or arrays of structs that contain arrays.

### Slices

:::caution

This feature is experimental. You should expect it to change in future versions,
cause unexpected behavior, or simply not work at all.

:::

A slice is a dynamically-sized view into a sequence of elements. They can be resized at runtime, but because they don't own the data, they cannot be returned from a circuit. You can treat slices as arrays without a constrained size.

Slices are part of the [noir standard library](../standard_library/slice_methods) so you need to import the respective module in order to work with it. For example:

```rust
use dep::std::slice;

fn main() -> pub Field {
    let mut slice: [Field] = [0; 2];

    let mut new_slice = slice.push_back(6);
    new_slice.len()
}
```

### Vectors

:::caution

This feature is experimental. You should expect it to change in future versions,
cause unexpected behavior, or simply not work at all.

:::

A vector is a collection type similar to Rust's Vector type. It's convenient way to use slices as mutable arrays.

Example:

```rust
use std::collections::vec::Vec;

let mut vector: Vec<Field> = Vec::new();
for i in 0..5 {
    vector.push(i);
}
assert(vector.len() == 5);
```

### Tuples

A tuple collects multiple values like an array, but with the added ability to collect values of
different types:

```rust
fn main() {
    let tup: (u8, u64, Field) = (255, 500, 1000);
}
```

One way to access tuple elements is via destructuring using pattern matching:

```rust
fn main() {
    let tup = (1, 2);

    let (one, two) = tup;

    let three = one + two;
}
```

Another way to access tuple elements is via direct member access, using a period (`.`) followed by
the index of the element we want to access. Index `0` corresponds to the first tuple element, `1` to
the second and so on:

```rust
fn main() {
    let tup = (5, 6, 7, 8);

    let five = tup.0;
    let eight = tup.3;
}
```

### Structs

A struct also allows for grouping multiple values of different types. Unlike tuples, we can also
name each field.

> **Note:** The usage of _field_ here refers to each element of the struct and is unrelated to the
> field type of Noir.

Defining a struct requires giving it a name and listing each field within as `<Key>: <Type>` pairs:

```rust
struct Animal {
    hands: Field,
    legs: Field,
    eyes: u8,
}
```

An instance of a struct can then be created with actual values in `<Key>: <Value>` pairs in any
order. Struct fields are accessible using their given names:

```rust
fn main() {
    let legs = 4;

    let dog = Animal {
        eyes: 2,
        hands: 0,
        legs,
    };

    let zero = dog.hands;
}
```

Structs can also be destructured in a pattern, binding each field to a new variable:

```rust
fn main() {
    let Animal { hands, legs: feet, eyes } = get_octopus();

    let ten = hands + feet + eyes as u8;
}

fn get_octopus() -> Animal {
    let octopus = Animal {
        hands: 0,
        legs: 8,
        eyes: 2,
    };

    octopus
}
```

The new variables can be bound with names different from the original struct field names, as
showcased in the `legs --> feet` binding in the example above.

:::note
You can use Structs as inputs to the `main` function, but you can't output them
:::

### Type Aliases

A type alias is a new name for an existing type. Type aliases are declared with the keyword `type`:

```rust
type Id = u8;

fn main() {
    let id: Id = 1;
    let zero: u8 = 0;
    assert(zero + 1 == id);
}
```

Type aliases can also be used with [generics](./06_generics.md):

```rust
type Id<Size> = Size;

fn main() {
    let id: Id<u32> = 1;
    let zero: u32 = 0;
    assert(zero + 1 == id);
}
```

### BigInt

You can acheive BigInt functionality using the [Noir BigInt](https://github.com/shuklaayush/noir-bigint) library.

## References

Noir supports first-class references. References are a bit like pointers: they point to a specific address that can be followed to access the data stored at that address. You can use Rust-like syntax to use pointers in Noir: the `&` operator references the variable, the `*` operator dereferences it.

Example:

```rust
fn main() {
    let mut x = 2;

    // you can reference x as &mut and pass it to multiplyBy2
    multiplyBy2(&mut x);
}

// you can access &mut here
fn multiplyBy2(x: &mut Field) {
    // and dereference it with *
    *x = *x * 2;
}
```
