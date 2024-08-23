---
title: Quoted
---

`std::meta::quoted` contains methods on the built-in `Quoted` type which represents
quoted token streams and is the result of the `quote { ... }` expression.

## Methods

### as_expr

#include_code as_expr noir_stdlib/src/meta/quoted.nr rust

Parses the quoted token stream as an expression. Returns `None` if
the expression failed to parse.

Example:

```rust
comptime {
    let add: Expr = quote { 1 + 2 }.as_expr().unwrap();

    let (_one, op, _two) = add.as_binary_op().unwrap();
    assert(op.is_add());
}
```

### as_module

#include_code as_module noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a module path leading to the name of a module.
Returns `None` if the module isn't found or the token stream cannot be parsed as a path.

Example:

```rust
mod foo {
    mod bar { ... }
}

comptime {
    let my_mod = quote { foo::bar }.as_module().unwrap();
    assert_eq(my_mod.name(), quote { bar });
}
```

### as_trait_constraint

#include_code as_trait_constraint noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a trait constraint (without an object type).
Note that this function panics instead of returning None if the token
stream does not parse and resolve to a valid trait constraint.

Example:

```rust
comptime {
    let eq = quote { Eq }.as_trait_constraint();
    let i32_type = quote { i32 }.as_type();
    assert(i32_type.implements(eq));
}
```

### as_type

#include_code as_type noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a resolved type. Panics if the token
stream doesn't parse to a type or if the type isn't a valid type in scope.

```rust
comptime {
    let eq = quote { Eq }.as_trait_constraint();
    let i32_type = quote { i32 }.as_type();
    assert(i32_type.implements(eq));
}
```

## Trait Implementations

```rust
impl Eq for Quoted
```
