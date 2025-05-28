---
title: Shadowing
sidebar_position: 12
---

Noir allows for inheriting variables' values and re-declaring them with the same name similar to Rust, known as shadowing.

For example, the following function is valid in Noir:

```rust
fn main() {
    let x = 5;

    {
        let x = x * 2;
        assert (x == 10);
    }

    assert (x == 5);
}
```

In this example, a variable x is first defined with the value 5.

The local scope that follows shadows the original x, i.e. creates a local mutable x based on the value of the original x. It is given a value of 2 times the original x.

When we return to the main scope, x once again refers to just the original x, which stays at the value of 5.

## Temporal mutability

One way that shadowing is useful, in addition to ergonomics across scopes, is for temporarily mutating variables.

```rust
fn main() {
    let age = 30;
    // age = age + 5; // Would error as `age` is immutable by default.

    let mut age = age + 5; // Temporarily mutates `age` with a new value.

    let age = age; // Locks `age`'s mutability again.

    assert (age == 35);
}
```
