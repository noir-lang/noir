---
title: Quoted
description: Token streams produced by `quote { ... }`—parse as expressions, modules, types, and inspect raw tokens.
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

### location

#include_code location noir_stdlib/src/meta/quoted.nr rust

Returns the source location spanning the tokens of this stream, or `Option::none()` if
the stream is empty.

### tokens

#include_code tokens noir_stdlib/src/meta/quoted.nr rust

Returns a vector of the individual tokens that form this token stream.

## Trait Implementations

```rust
impl Eq for Quoted
impl Hash for Quoted
```

## Security

A `Quoted` value returned by a comptime function from an untrusted dependency is added, when unquoted,
to the program's source as if you had written it yourself. It can therefore introduce arbitrary items,
expressions, or trait impls into the crate that calls it, including code that the caller did not write
or would not approve.

When pulling in comptime code from a dependency you do not control:

- Treat any `Quoted` it returns to you, and any attribute macro it provides, as code you are about
  to vendor into your own crate.
- Use `nargo expand` to view the program after macro expansion, and review the result before
  deployment. The expanded source is the code that actually compiles into your circuit.

See also the [Security considerations](../../../language/comptime.md#security-considerations) section
of the comptime documentation.
