---
title: TraitDefinition
description: Work with trait definitions at compile time—convert to trait constraints and inspect basic properties.
---

`std::meta::trait_def` contains methods on the built-in `TraitDefinition` type. This type
represents trait definitions such as `trait Foo { .. }` at the top-level of a program.

## Methods

### as_trait_constraint

#include_code as_trait_constraint noir_stdlib/src/meta/trait_def.nr rust

Converts this trait into a trait constraint. If there are any generics on this
trait, they will be kept as-is without instantiating or replacing them.

### location

#include_code location noir_stdlib/src/meta/trait_def.nr rust

Returns the source [`Location`](./location.md) where this trait is defined.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the trait.

## Trait Implementations

```rust
impl Eq for TraitDefinition
impl Hash for TraitDefinition
```
