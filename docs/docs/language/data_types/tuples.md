---
title: Tuples
description:
  Dive into the Tuple data type in Noir. Understand its methods, practical examples, and best practices for efficiently using Tuples in your Noir code.
keywords:
  [
    noir,
    tuple type,
    methods,
    examples,
    multi-value containers,
  ]
---

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

## Comparing tuples

Tuples of up to 12 elements can be compared, as long as every element type is itself comparable.

Equality (`==`) and inequality (`!=`) require each element type to implement `Eq`. Two tuples are
equal when all of their corresponding elements are equal:

```rust
fn main() {
    assert((1, 2, 3) == (1, 2, 3));
    assert((1, 2, 3) != (1, 2, 4));
}
```

The ordering operators (`<`, `<=`, `>`, `>=`) require each element type to implement `Ord`. Tuples
are ordered lexicographically: the first elements are compared, and only if they are equal are the
next elements compared, and so on:

```rust
fn main() {
    assert((1, 2) < (1, 3)); // first elements equal, second decides
    assert((1, 5) < (2, 0)); // first element decides, second is ignored
    assert((1, 2, 3) <= (1, 2, 3));
}
```
