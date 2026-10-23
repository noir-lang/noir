---
title: TraitDefinition
description: Work with trait definitions at compile time—convert to trait constraints and inspect basic properties.
---

`std::meta::trait_def` contains methods on the built-in `TraitDefinition` type. This type
represents trait definitions such as `trait Foo { .. }` at the top-level of a program.

## Methods

### as_trait_constraint

```rust title="as_trait_constraint" showLineNumbers 
pub comptime fn as_trait_constraint(_self: Self) -> TraitConstraint {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/trait_def.nr#L34-L36" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/trait_def.nr#L34-L36</a></sub></sup>


Converts this trait into a trait constraint. If there are any generics on this
trait, they will be kept as-is without instantiating or replacing them.

### location

```rust title="location" showLineNumbers 
pub comptime fn location(self) -> Location {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/trait_def.nr#L61-L63" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/trait_def.nr#L61-L63</a></sub></sup>


Returns the source [`Location`](./location.md) where this trait is defined.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the trait.

## Trait Implementations

```rust
impl Eq for TraitDefinition
impl Hash for TraitDefinition
```
