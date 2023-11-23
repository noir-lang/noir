---
title: References
---

Noir supports first-class references. References are a bit like pointers: they point to a specific address that can be followed to access the data stored at that address. You can use Rust-like syntax to use pointers in Noir: the `&` operator references the variable, the `*` operator dereferences it.

Example:

```rust
fn main() {
    let mut x = 2;

    // you can reference x as &mut and pass it to multiplyBy2
    multiplyBy2(&mut x);
}

// you can access &mut here
fn multiplyBy2(x: &mut Field) {
    // and dereference it with *
    *x = *x * 2;
}
```
