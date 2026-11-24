---
title: Location
description: A source location at compile time, used to attach diagnostics (errors and warnings) to specific items or expressions.
---

`std::meta::location` contains the built-in `Location` type which represents a source location
in a Noir program. `Location` values are produced by other comptime types (`FunctionDefinition`,
`Module`, `TraitDefinition`, `TypeDefinition`, and `TypedExpr` all expose a `location` method)
and are typically passed to `std::meta::error` and `std::meta::warn` to attach a diagnostic to
a specific item or expression in the source program.

## Trait Implementations

```rust
impl Eq for Location
impl Hash for Location
```

Two `Location` values compare equal if and only if they refer to the same source span.
