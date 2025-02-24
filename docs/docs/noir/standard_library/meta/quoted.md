---
title: Quoted
---

`std::meta::quoted` contains methods on the built-in `Quoted` type which represents
quoted token streams and is the result of the `quote { ... }` expression.

## Methods

### as_expr

#include_code as_expr noir_stdlib/src/meta/quoted.nr rust

Parses the quoted token stream as an expression. Returns `Option::none()` if
the expression failed to parse.

Example:

#include_code as_expr_example test_programs/noir_test_success/comptime_expr/src/main.nr rust

### as_module

#include_code as_module noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a module path leading to the name of a module.
Returns `Option::none()` if the module isn't found or this token stream cannot be parsed as a path.

Example:

#include_code as_module_example test_programs/compile_success_empty/comptime_module/src/main.nr rust

### as_trait_constraint

#include_code as_trait_constraint noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a trait constraint (without an object type).
Note that this function panics instead of returning `Option::none()` if the token
stream does not parse and resolve to a valid trait constraint.

Example:

#include_code implements_example test_programs/compile_success_empty/comptime_type/src/main.nr rust

### as_type

#include_code as_type noir_stdlib/src/meta/quoted.nr rust

Interprets this token stream as a resolved type. Panics if the token
stream doesn't parse to a type or if the type isn't a valid type in scope.

#include_code implements_example test_programs/compile_success_empty/comptime_type/src/main.nr rust

### tokens

#include_code tokens noir_stdlib/src/meta/quoted.nr rust

Returns a slice of the individual tokens that form this token stream.

## Trait Implementations

```rust
impl Eq for Quoted
impl Hash for Quoted
```
