---
title: Constrain Statements
description:
  Learn about the constrain keyword in Noir, which can be used to explicitly constrain the predicate
  or comparison expression that follows to be true, and what happens if the expression is false at
  runtime.
keywords:
  [Noir programming language, constrain statement, predicate expression, comparison expression]
---

:::danger

In versions >=0.5.0 use the [`assert`](./04_assert.md) syntax. The `constrain` statement will be
maintained for some time for backwards compatibility but will be deprecated in the future.

:::

Noir includes a special keyword `constrain` which will explicitly constrain the predicate/comparison
expression that follows to be true. If this expression is false at runtime, the program will fail to
be proven.

### Constrain statement example

```rust
fn main(x : Field, y : Field) {
    constrain x == y;
}
```

The above snippet compiles because `==` is a predicate operation. Conversely, the following will not
compile:

```rust
fn main(x : Field, y : Field) {
    constrain x + y;
}
```

> The rationale behind this not compiling is due to ambiguity. It is not clear if the above should
> equate to `x + y == 0` or if it should check the truthiness of the result.
