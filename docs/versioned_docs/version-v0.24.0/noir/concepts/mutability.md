---
title: Mutability
description:
  Learn about mutable variables in Noir. Discover how
  to declare, modify, and use them in your programs.
keywords: [noir programming language, mutability in noir, mutable variables]
sidebar_position: 8
---

Variables in noir can be declared mutable via the `mut` keyword. Mutable variables can be reassigned
to via an assignment expression.

```rust
let x = 2;
x = 3; // error: x must be mutable to be assigned to

let mut y = 3;
let y = 4; // OK
```

The `mut` modifier can also apply to patterns:

```rust
let (a, mut b) = (1, 2);
a = 11; // error: a must be mutable to be assigned to
b = 12; // OK

let mut (c, d) = (3, 4);
c = 13; // OK
d = 14; // OK

// etc.
let MyStruct { x: mut y } = MyStruct { x: a };
// y is now in scope
```

Note that mutability in noir is local and everything is passed by value, so if a called function
mutates its parameters then the parent function will keep the old value of the parameters.

```rust
fn main() -> pub Field {
    let x = 3;
    helper(x);
    x // x is still 3
}

fn helper(mut x: i32) {
    x = 4;
}
```

## Non-local mutability

Non-local mutability can be achieved through the mutable reference type `&mut T`:

```rust
fn set_to_zero(x: &mut Field) {
    *x = 0;
}

fn main() {
    let mut y = 42;
    set_to_zero(&mut y);
    assert(*y == 0);
}
```

When creating a mutable reference, the original variable being referred to (`y` in this
example) must also be mutable. Since mutable references are a reference type, they must
be explicitly dereferenced via `*` to retrieve the underlying value. Note that this yields
a copy of the value, so mutating this copy will not change the original value behind the
reference:

```rust
fn main() {
    let mut x = 1;
    let x_ref = &mut x;

    let mut y = *x_ref;
    let y_ref = &mut y;

    x = 2;
    *x_ref = 3;

    y = 4;
    *y_ref = 5;

    assert(x == 3);
    assert(*x_ref == 3);
    assert(y == 5);
    assert(*y_ref == 5);
}
```

Note that types in Noir are actually deeply immutable so the copy that occurs when
dereferencing is only a conceptual copy - no additional constraints will occur.

Mutable references can also be stored within structs. Note that there is also
no lifetime parameter on these unlike rust. This is because the allocated memory
always lasts the entire program - as if it were an array of one element.

```rust
struct Foo {
    x: &mut Field
}

impl Foo {
    fn incr(mut self) {
        *self.x += 1;
    }
}

fn main() {
    let foo = Foo { x: &mut 0 };
    foo.incr();
    assert(*foo.x == 1);
}
```

In general, you should avoid non-local & shared mutability unless it is needed. Sticking
to only local mutability will improve readability and potentially improve compiler optimizations as well.
