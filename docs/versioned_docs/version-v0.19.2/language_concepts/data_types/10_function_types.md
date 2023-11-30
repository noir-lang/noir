---
title: Function types
---

Noir supports higher-order functions. The syntax for a function type is as follows:

```rust
fn(arg1_type, arg2_type, ...) -> return_type
```

Example:

```rust
fn assert_returns_100(f: fn() -> Field) { // f takes no args and returns a Field
    assert(f() == 100);
}

fn main() {
    assert_returns_100(|| 100); // ok
    assert_returns_100(|| 150); // fails
}
```

A function type also has an optional capture environment - this is necessary to support closures.
See [Lambdas](../08_lambdas.md) for more details.
