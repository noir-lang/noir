---
title: Mutability
description:
  Learn about mutable variables, constants, and globals in Noir programming language. Discover how
  to declare, modify, and use them in your programs.
keywords: [noir programming language, mutability in noir, mutable variables, constants, globals]
---

Variables in Noir can be declared mutable via the `mut` keyword. Mutable variables can be reassigned
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
let MyStruct { x: mut y } = MyStruct { x: a }
// y is now in scope
```

Note that mutability in noir is local and everything is passed by value, so if a called function
mutates its parameters then the parent function will keep the old value of the parameters.

```rust
fn main() -> Field {
    let x = 3;
    helper(x);
    x // x is still 3
}

fn helper(mut x: i32) {
    x = 4;
}
```

## Comptime Values

Comptime values are values that are known at compile-time. This is different to a witness
which changes per proof. If a comptime value that is being used in your program is changed, then your
circuit will also change.

Comptime is slightly different from Rust's `const`. Namely, it is a bit more flexible in that normal functions can accept comptime parameters. For example, this is used to verify an array index is known at compile-time. Note that the "known at compile-time" here means "known after function inlining is performed while optimizing the program" and not "known during type-checking."

Below we show how to declare a comptime value:

```rust
fn main() {
    let a: comptime Field = 5;

    // `comptime Field` can also be inferred:
    let a = 5;
}
```

Note that variables declared as mutable may not be comptime:

```rust
fn main() {
    // error: Cannot mark a comptime type as mutable
    let mut a: comptime Field = 5;

    // a inferred as a private Field here
    let mut a = 5;
}
```

## Globals

Noir also supports global variables. However, they must be compile-time variables. If `comptime` is
not explicitly written in the type annotation the compiler will implicitly specify the declaration
as compile-time. They can then be used like any other compile-time variable inside functions. The
global type can also be inferred by the compiler entirely. Globals can also be used to specify array
annotations for function parameters and can be imported from submodules.

```rust
global N: Field = 5; // Same as `global N: comptime Field = 5`

fn main(x : Field, y : [Field; N]) {
    let res = x * N;

    assert(res == y[0]);

    let res2 = x * mysubmodule::N;
    assert(res != res2);
}

mod mysubmodule {
    use dep::std;

    global N: Field = 10;

    fn my_helper() -> comptime Field {
        let x = N;
        x
    }
}
```

## Why only local mutability?

Witnesses in a proving system are immutable in nature. Noir aims to _closely_ mirror this setting
without applying additional overhead to the user. Modeling a mutable reference is not as
straightforward as on conventional architectures and would incur some possibly unexpected overhead.
