---
title: Global Variables
description:
  Learn about global variables in Noir. Discover how
  to declare, modify, and use them in your programs.
keywords: [noir programming language, globals, global variables, constants]
sidebar_position: 8
---

## Globals


Noir supports global variables. The global's type can be inferred by the compiler entirely:

```rust
global N = 5; // Same as `global N: Field = 5`

global TUPLE = (3, 2);

fn main() {
    assert(N == 5);
    assert(N == TUPLE.0 + TUPLE.1);
}
```

:::info

Globals can be defined as any expression, so long as they don't depend on themselves - otherwise there would be a dependency cycle! For example:

```rust
global T = foo(T); // dependency error
```

:::


If they are initialized to a literal integer, globals can be used to specify an array's length:

```rust
global N: Field = 2;

fn main(y : [Field; N]) {
    assert(y[0] == y[1])
}
```

A global from another module can be imported or referenced externally like any other name:

```rust
global N = 20;

fn main() {
    assert(my_submodule::N != N);
}

mod my_submodule {
    global N: Field = 10;
}
```

When a global is used, Noir replaces the name with its definition on each occurrence.
This means globals defined using function calls will repeat the call each time they're used:

```rust
global RESULT = foo();

fn foo() -> [Field; 100] { ... }
```

This is usually fine since Noir will generally optimize any function call that does not
refer to a program input into a constant. It should be kept in mind however, if the called
function performs side-effects like `println`, as these will still occur on each use.
