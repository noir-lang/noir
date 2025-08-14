---
title: Attributes
description:
  Learn how to use attributes in Noir. Attributes are metadata that can be applied to data types,
  functions and variables.
keywords: [Noir, attributes]
sidebar_position: 15
---

Attributes are metadata that can be applied to data types, functions, and some statements and expressions,
using the following syntax: `#[attribute(value)]`.

### `allow(dead_code)`

When applied to a data type or function, the compiler won't produce a warning if the data type or function
ends up being unused.

Example:

```rust
#[allow(dead_code)]
struct Unused {}
```

### `allow(unused_variables)`

When applied on a `let` statement, the compiler won't produce a warning if the variable ends up being unused.

Example:

```rust
fn main() {
    #[allow(unused_variables)]
    let unused = 1;
}
```

### `builtin`

When applied to a function, indicates that the function is implemented by the compiler, for efficiency purposes.

### `deprecated`

Marks a function as _deprecated_. Calling the function will generate a warning: `warning: use of deprecated function`

Example:

```rust
#[deprecated]
fn slow_function() {}
```

The attribute takes an optional string which will be used as the deprecation message:
`#[deprecated("use some other function")]`

Example:

```rust
#[deprecated("use fast_function")]
fn slow_function() {}

fn fast_function() {}
```

### `field`

Can be used on functions to enable conditional compilation of code depending on the field size.

The field attribute defines which field the function is compatible for. The function is conditionally compiled, under the condition that the field attribute matches the Noir native field.
The field can be defined implicitly, by using the name of the elliptic curve usually associated to it - for instance bn254, bls12_381 - or explicitly by using the field (prime) order, in decimal or hexadecimal form.
As a result, it is possible to define multiple versions of a function with each version specialized for a different field attribute. This can be useful when a function requires different parameters depending on the underlying elliptic curve.

Example: we define the function `foo()` three times below. Once for the default Noir bn254 curve, once for the field $\mathbb F_{23}$, which will normally never be used by Noir, and once again for the bls12_381 curve.

```rust
#[field(bn254)]
fn foo() -> u32 {
    1
}

#[field(23)]
fn foo() -> u32 {
    2
}

// This commented code would not compile as foo would be defined twice because it is the same field as bn254
// #[field(21888242871839275222246405745257275088548364400416034343698204186575808495617)]
// fn foo() -> u32 {
//     2
// }

#[field(bls12_381)]
fn foo() -> u32 {
    3
}
```

If the field name is not known to Noir, it will discard the function. Field names are case insensitive.

### `fuzz`

Marks the functions for fuzzing. See [Fuzzer](../../tooling/fuzzer.md) for more details.

### `oracle`

Mark a function as _oracle_; meaning it is an external unconstrained function, implemented in noir_js. See [Unconstrained](./unconstrained.md) and [NoirJS](../../reference/NoirJS/noir_js/index.md) for more details.

### `test`

Marks the function as a unit test. See [Tests](../../tooling/tests.md) for more details.