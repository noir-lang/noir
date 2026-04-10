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

### `allow(unused_mut)`

When applied on a `let` statement, the compiler won't produce a warning if the variable is `mut` but
is never mutated.

Example:

```rust
fn main() {
    #[allow(unused_mut)]
    let mut never_mutated = 1;
    println(never_mutated);
}
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

To change the warning into a hard error, `deny` can be specified on the attribute:

```rust
#[deprecated(deny, "don't use this!")]
fn broken() {}
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

Mark a function as _oracle_; meaning it is an external unconstrained function, implemented in noir_js. See [Unconstrained](./unconstrained.md) for more details.

### `fold`

Marks a function for ACIR fold optimization. The compiler will generate a separate circuit for this function which is then recursively verified at runtime. This can reduce total circuit size when a function is called multiple times or contains a large number of constraints.

Example:

```rust
#[fold]
fn expensive_computation(x: Field) -> Field {
    // ... many constraints ...
    x
}
```

### `inline_always`

Forces the compiler to always inline this function at its call sites. This can improve performance by avoiding function call overhead, at the cost of increased circuit size.

Since constrained calls are always inlined, this will only ever have any effect in unconstrained code.

Example:

```rust
#[inline_always]
fn small_helper(x: Field) -> Field {
    x + 1
}
```

### `inline_never`

Prevents the compiler from inlining this function. This can be useful to keep circuit size manageable when a function is called from many places.

Since constrained calls are always inlined, this will only ever have any effect in unconstrained code.

Example:

```rust
#[inline_never]
fn large_function(x: Field) -> Field {
    // ... complex logic ...
    x
}
```

### `no_predicates`

Disables predicate optimization for this function's ACIR output. This can be useful when the predicate optimization would produce incorrect results for certain foreign function patterns.

This will have no effect in unconstrained code.

### `export`

Marks a function for export in compiled artifacts. This is primarily used inside `contract` blocks to indicate that a function should be accessible externally.

### `must_use`

Produces a warning if the return value of this function call is unused. Optionally takes a message string explaining why the value should be used.

Example:

```rust
#[must_use]
fn important_result() -> Field {
    42
}

#[must_use = "the new vector is returned, the original is not modified"]
fn push(vec: [Field], value: Field) -> [Field] {
    // ...
}
```

### `abi`

Tags a struct or global for inclusion in a contract's ABI with the given tag name. Used inside `contract` blocks to indicate which types should appear in the compiled artifact's interface.

Example:

```rust
#[abi(events)]
struct Transfer {
    from: Field,
    to: Field,
    amount: Field,
}
```

### `contract_library_method`

Marks a function inside a `contract` block as a helper method rather than a contract entry point. Functions with this attribute will not be included as callable endpoints in the compiled contract artifact.

### `varargs`

Allows a comptime attribute function to accept a variable number of arguments. See [Compile-time Code](./comptime.md) for more details.

### `derive`

Derives trait implementations for a struct using comptime macros. Multiple traits can be derived at once by separating them with commas. See [Compile-time Code](./comptime.md#example-derive) for more details.

Example:

```rust
#[derive(Default, Eq, Ord)]
struct MyStruct {
    field1: u32,
    field2: Field,
}
```

### `test`

Marks the function as a unit test. See [Tests](../../tooling/tests.md) for more details.

### Inner Attributes

Inner attributes apply to the enclosing module rather than the item that follows. They use the syntax `#![attribute]`:

```rust
#![allow(unused_variables)]
```

Inner attributes support `#[allow(...)]`, `#[deprecated]`, and custom meta attributes, using the same syntax as outer attributes but prefixed with `!`.
