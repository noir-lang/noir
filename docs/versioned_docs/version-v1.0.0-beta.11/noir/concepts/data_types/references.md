---
title: References
sidebar_position: 9
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

References do have limitations. Mutable references to array elements are not supported.

For example, the following code snippet:
```rust
fn foo(x: &mut u32) {
    *x += 1;
}
fn main() {
    let mut state: [u32; 4] = [1, 2, 3, 4];
    foo(&mut state[0]);
    assert_eq(state[0], 2); // expect:2 got:1
}
```
Will error with the following:
```
error: Mutable references to array elements are currently unsupported
  ┌─ src/main.nr:6:18
  │
6 │         foo(&mut state[0]);
  │                  -------- Try storing the element in a fresh variable first
  │
```
