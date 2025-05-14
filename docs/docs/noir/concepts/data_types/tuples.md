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
sidebar_position: 7
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

When destructuring via pattern matching, `..` can be used to exclude a range of elements:

```rust
fn main() {
   let tup = (1, 2, 3, 4, 5, 6, 7, 8);

   let (one, two, .., six, seven, eight) = tup;
   let (one, two, ..) = tup;
   let (.., five, six, seven, eight) = tup;
   let (..) = tup;
}
```