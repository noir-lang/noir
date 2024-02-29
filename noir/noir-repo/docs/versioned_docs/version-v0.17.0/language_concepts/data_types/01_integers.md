---
title: Integers
description:
  Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords:
  [
    noir,
    integer types,
    methods,
    examples,
    arithmetic,
  ]
---

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
