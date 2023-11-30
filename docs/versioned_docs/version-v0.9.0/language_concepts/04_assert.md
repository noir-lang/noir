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
be proven.

### Example

```rust
fn main(x : Field, y : Field) {
    assert(x == y);
}
```

The above snippet compiles because `==` is a predicate operation. Conversely, the following will not
compile:

```rust
// INCORRECT

fn main(x : Field, y : Field) {
    assert(x + y);
}
```

> The rationale behind this not compiling is due to ambiguity. It is not clear if the above should
> equate to `x + y == 0` or if it should check the truthiness of the result.
