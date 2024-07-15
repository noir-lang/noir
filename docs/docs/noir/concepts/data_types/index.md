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

## Type Aliases

A type alias is a new name for an existing type. Type aliases are declared with the keyword `type`:

```rust
type Id = u8;

fn main() {
    let id: Id = 1;
    let zero: u8 = 0;
    assert(zero + 1 == id);
}
```

Type aliases can also be used with [generics](../generics.md):

```rust
type Id<Size> = Size;

fn main() {
    let id: Id<u32> = 1;
    let zero: u32 = 0;
    assert(zero + 1 == id);
}
```

Type aliases can even refer to other aliases. An error will be issued if they form a cycle:

```rust
// Ok!
type A = B;
type B = Field;

type Bad1 = Bad2;

// error: Dependency cycle found
type Bad2 = Bad1;
//   ^^^^^^^^^^^ 'Bad2' recursively depends on itself: Bad2 -> Bad1 -> Bad2
```

## Wildcard Type
Noir can usually infer the type of the variable from the context, so specifying the type of a variable is only required when it cannot be inferred. However, specifying a complex type can be tedious, especially when it has multiple generic arguments. Often some of the generic types can be inferred from the context, and Noir only needs a hint to properly infer the other types. We can partially specify a variable's type by using `_` as a marker, indicating where we still want the compiler to infer the type.

```rust
let a: [_; 4] = foo(b);
```
 

### BigInt

You can achieve BigInt functionality using the [Noir BigInt](https://github.com/shuklaayush/noir-bigint) library.
