---
title: Mutability
description:
  Learn about mutable variables, constants, and globals in Noir programming language. Discover how
  to declare, modify, and use them in your programs.
keywords: [noir programming language, mutability in noir, mutable variables, constants, globals]
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

:::warning

The 'comptime' keyword was removed in version 0.10. The comptime keyword and syntax are currently still kept and parsed for backwards compatibility, but are now deprecated and will issue a warning when used. `comptime` has been removed because it is no longer needed for accessing arrays.

:::

## Globals

Noir also supports global variables. However, they must be known at compile-time. The global type can also be inferred by the compiler entirely. Globals can also be used to specify array
annotations for function parameters and can be imported from submodules.

```rust
global N: Field = 5; // Same as `global N: Field = 5`

fn main(x : Field, y : [Field; N]) {
    let res = x * N;

    assert(res == y[0]);

    let res2 = x * mysubmodule::N;
    assert(res != res2);
}

mod mysubmodule {
    use dep::std;

    global N: Field = 10;

    fn my_helper() -> Field {
        let x = N;
        x
    }
}
```

## Why only local mutability?

Witnesses in a proving system are immutable in nature. Noir aims to _closely_ mirror this setting
without applying additional overhead to the user. Modeling a mutable reference is not as
straightforward as on conventional architectures and would incur some possibly unexpected overhead.
