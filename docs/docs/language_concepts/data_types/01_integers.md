---
title: Integers
description: Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords: [noir, integer types, methods, examples, arithmetic]
---

An integer type is a range constrained field type. The Noir frontend currently supports unsigned,
arbitrary-sized integer types.

When an integer is defined in Noir without a specific type, it will default to `Field`. The one exception is for loop indices which default to `u64` since comparisons on `Field`s are not possible.

An integer type is specified first with the letter `u`, indicating its unsigned nature, followed by
its length in bits (e.g. `8`). For example, a `u8` variable can store a value in the range of
$\\([0,2^{8}-1]\\)$.

> **Note:** The default proving backend supports both even (e.g. _u2_, _u32_) and odd (e.g. _u3_, _u127_) sized integer types.

Taking a look of how the type is used:

```rust
fn main(x : Field, y : u8) {
    let z = x as u8 + y;
    assert (z > 0);
}
```

Note that `x`, `y` and `z` are all private values in this example, where `x` is a field while `y` and `z`
are unsigned 32-bit integers.

If `y` or `z` exceeds the range $\\([0,2^{8}-1]\\)$, proofs created
will be rejected by the verifier.

For example, attempting to prove the above code with the following inputs:

```toml
x = "1"
y = "256"
```

Would result in:

```
$ nargo prove
The parameter y is expected to be a Integer { sign: Unsigned, width: 8 } but found incompatible value Field(2⁸)
```
