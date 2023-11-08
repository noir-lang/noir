---
title: Assert Function
description:
  Learn about the assert function in Noir, which can be used to explicitly constrain the predicate or
  comparison expression that follows to be true, and what happens if the expression is false at
  runtime.
keywords: [Noir programming language, assert statement, predicate expression, comparison expression]
---

Noir includes a special `assert` function which will explicitly constrain the predicate/comparison
expression that follows to be true. If this expression is false at runtime, the program will fail to
be proven. Example:

```rust
fn main(x : Field, y : Field) {
    assert(x == y);
}
```

You can optionally provide a message to be logged when the assertion fails:

```rust
assert(x == y, "x and y are not equal");
```

> Assertions only work for predicate operations, such as `==`. If there's any ambiguity on the operation, the program will fail to compile. For example, it is unclear if `assert(x + y)` would check for `x + y == 0` or simply would return `true`.
