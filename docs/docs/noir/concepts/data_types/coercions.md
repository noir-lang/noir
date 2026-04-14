---
title: Type Casting and Coercions
description:
  Noir's explicit type casts and implicit type coercions
keywords:
  [
    noir,
    types,
    coercions,
    casts,
    as,
  ]
sidebar_position: 11
---

## Type Casting

You can explicitly convert between numeric types using the `as` keyword:

```rust
let x: u32 = 100;
let y: u8 = x as u8;       // Cast u32 to u8 (truncates if value exceeds u8 range)
let z: Field = x as Field;  // Cast u32 to Field
```

### Valid casts

The `as` keyword can be used to cast between the following types:

| From | To | Notes |
|------|----|-------|
| Any integer | Any other integer | Truncates to fit the target bit size |
| Any unsigned integer | `Field` | Always safe, no truncation |
| `Field` | Any integer | Truncates to fit the target bit size |

### Restrictions

- **Signed integers cannot be cast to `Field`** -- this will produce a compiler error. Convert to an unsigned integer first if needed.
- **Non-numeric types** (arrays, strings, structs, tuples, etc.) cannot be cast to or from numeric types.
- **Casting to `bool`** is not supported. Compare with zero instead: `value != 0`

### Example

```rust
fn main() {
    let big: u32 = 300;
    let small = big as u8;    // Truncates: 300 does not fit in u8
    
    let field_val: Field = 42;
    let as_u64 = field_val as u64;
    
    let unsigned: u32 = 5;
    let as_field = unsigned as Field;  // Ok: unsigned to Field
    
    // let signed: i32 = -1;
    // let bad = signed as Field;  // Error: cannot cast signed integer to Field
}
```

### Performance note

Integer casts generate range check constraints in the compiled circuit. Using `Field` values directly where possible can avoid these extra constraints.

See [Security Considerations for Field Arithmetic](./fields.md#security-considerations-for-field-arithmetic) to determine if the tradeoffs are worth the cheaper casting & arithmetic costs of using field arithmetic directly.

## Type Coercions

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
fn requires_vector(_vector: [Field]) {}
comptime fn requires_ct_string(_s: CtString) {}

fn main() {
    let array: [Field; 4] = [1, 2, 3, 4];

    // Ok - array is converted to a vector
    requires_vector(array);
    // equivalent to:
    requires_vector(array.as_vector());

    // coerce a constrained function to an unconstrained one:
    let f: unconstrained fn([Field]) = requires_vector;

    comptime {
        // Passing a str<6> where a CtString is expected
        requires_ct_string("hello!")
    }
}
```
