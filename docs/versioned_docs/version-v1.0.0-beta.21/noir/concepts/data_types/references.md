---
title: References
description: Learn about reference types in Noir, including immutable references (&T) and mutable references (&mut T).
keywords: [noir, references, mutable references, immutable references, borrowing]
sidebar_position: 9
---

Noir supports first-class references. References point to a specific address that can be followed
to access the data stored at that address. The `&` operator references a variable and the `*` operator
dereferences it.

References come in two forms:
- **Immutable references** (`&T`) — read-only access to a value.
- **Mutable references** (`&mut T`) — read and write access to a value.

## Mutable References

Mutable references allow a called function to mutate a value owned by the caller:

```rust
fn main() {
    let mut x = 2;

    // you can reference x as &mut and pass it to multiplyBy2
    increment(&mut x);

    assert_eq(x, 3);
}

// you can access &mut here
fn increment(x: &mut Field) {
    // and dereference it with *
    *x += 1;
}
```

When creating a mutable reference, the original variable must be declared `mut`.

## Immutable References

Immutable references provide read-only access to a value and can be created with `&`:

```rust
fn sum(values: &[Field; 3]) -> Field {
    // Auto-dereferences: you can index directly through the reference
    values[0] + values[1] + values[2]
}

fn main() {
    let data = [1, 2, 3];
    let total = sum(&data);
    assert(total == 6);
}
```

Unlike mutable references, creating an immutable reference does not require the original variable to be declared `mut`.

You cannot write through an immutable reference. Attempting to do so is a compile error:

```rust
fn main() {
    let s = [0];
    let ps = &s;
    ps[0] = 1; // Error: `ps` is a `&` reference, so it cannot be written to
}
```

### Immutable Self References

Methods can take `&self` as a shorthand for `self: &Self`. Calling a `&self` method
does not require the variable to be mutable:

```rust
struct Point {
    x: Field,
    y: Field,
}

impl Point {
    fn sum(&self) -> Field {
        self.x + self.y
    }
}

fn main() {
    let p = Point { x: 1, y: 2 };
    assert(p.sum() == 3);
}
```

### Performance in Unconstrained Code

References only affect performance in unconstrained code. In constrained code, references
have no performance impact at all.

In unconstrained code, arrays use reference counting with copy-on-write semantics. Passing an array
by value increments its reference count. Passing it by immutable reference does *not* increment
the reference count.

When arrays are mutated in unconstrained code and the reference count is not 1, they are cloned.
Thus, passing around arrays by reference avoids reference count increases and can lead to fewer
clones at runtime.

If you are writing helper functions which are unconstrained or may be run in an unconstrained
context that only need to read an array, prefer taking `&[T; N]` over `[T; N]` to avoid the
reference-count overhead and to possibly reduce clones later on if the array is ever mutated.

### References Exclusivity and Lifetimes

Unlike Rust, Noir does **not** enforce reference exclusivity. An immutable reference
(`&T`) and a mutable reference (`&mut T`) to the same value can exist at the same time.
Noir also does not have lifetimes on references. Instead, referenced memory lasts for
the entire program.

### References to Array Elements

Mutable references to array elements are not supported:

```rust
fn foo(x: &mut u32) {
    *x += 1;
}
fn main() {
    let mut state: [u32; 4] = [1, 2, 3, 4];
    foo(&mut state[0]);
    assert_eq(state[0], 2);
}
```

The above will error with:

```
error: Mutable references to array elements are currently unsupported
  ┌─ src/main.nr:6:18
  │
6 │         foo(&mut state[0]);
  │                  -------- Try storing the element in a fresh variable first
  │
```
