---
title: Type Coercions
description:
  Noir's various type coercions
keywords:
  [
    noir,
    types,
    coercions,
    casts,
  ]
sidebar_position: 11
---

When one type is required in Noir code but a different type is given, the compiler will typically issue
a type error. There are a few cases however where the compiler will instead automatically perform a
type coercion. These are typically limited to a few type pairs where converting from one to the other
will not sacrifice performance or correctness. Currently, Noir will will try to perform the following
type coercions:

| Actual Type    | Expected Type               |
| -------------- | --------------------------- |
| `[T; N]`       | `[T]`                       |
| `fn(..) -> R`  | `unconstrained fn(..) -> R` |
| `str<N>`       | `CtString`                  |
| `fmtstr<N, T>` | `CtString`                  |
| `&mut T`       | `&T`                        |

Note that:
- Conversions are only from the actual type to the expected type, never the other way around.
- Conversions are only performed on the outermost type, they're never performed within a nested type.
- `CtString` is a compile-time only type, so this conversion is only valid in [comptime code](../../concepts/comptime.md).
- `&T` requires the experimental `-Zownership` flag to be enabled.

Examples:
```rust
fn requires_slice(_slice: [Field]) {}
comptime fn requires_ct_string(_s: CtString) {}

fn main() {
    let array: [Field; 4] = [1, 2, 3, 4];

    // Ok - array is converted to a slice
    requires_slice(array);
    // equivalent to:
    requires_slice(array.as_slice());

    // coerce a constrained function to an unconstrained one:
    let f: unconstrained fn([Field]) = requires_slice;

    comptime {
        // Passing a str<6> where a CtString is expected
        requires_ct_string("hello!")
    }
}
```
