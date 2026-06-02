---
title: Global Variables
description:
  Learn about global variables in Noir. Discover how
  to declare, modify, and use them in your programs.
keywords: [noir programming language, globals, global variables, constants]
---

## Globals


Noir supports global variables. The global's type must be specified by the user:

```rust
global N: Field = 5;

global TUPLE: (Field, Field) = (3, 2);

fn main() {
    assert(N == 5);
    assert(N == TUPLE.0 + TUPLE.1);
}
```

:::info

Globals can be defined as any expression, so long as they don't depend on themselves - otherwise there would be a dependency cycle! For example:

```rust
global T: u32 = foo(T); // dependency error
```

:::


If they are initialized to a literal integer, globals can be used to specify an array's length:

```rust
global N: u32 = 2;

fn main(y : [Field; N]) {
    assert(y[0] == y[1])
}
```

A global from another module can be imported or referenced externally like any other name:

```rust
global N: Field = 20;

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
global RESULT: [Field; 100] = foo();

fn foo() -> [Field; 100] { ... }
```

This is usually fine since Noir will generally optimize any function call that does not
refer to a program input into a constant. It should be kept in mind however, if the called
function performs side-effects like `println`, as these will still occur on each use.

### Visibility

By default, like functions, globals are private to the module they exist in. You can use `pub`
to make the global public or `pub(crate)` to make it public to just its crate:

```rust
// This global is now public
pub global N: u32 = 5;
```

### Overriding globals from the command line

A global's value can be overridden at compile time with the `--define` (`-D`) flag, ignoring
its declared initializer:

```bash
nargo compile -D N=256
```

```rust
global N: u32 = 100; // compiled as 256 with the flag above
```

The flag can be passed multiple times (`-D A=1 -D B=2`) and is matched against globals by name.
Only `bool`, `Field`, and integer-typed globals can be overridden; supplying a malformed value,
a value that does not fit the global's type, or a name with an unsupported type is an error,
while a name that matches no global is ignored.

This is useful for producing differently-sized circuits from a single codebase — for example
compiling "small", "medium", and "large" variants by overriding a size global, instead of
maintaining a separate package for each size.
