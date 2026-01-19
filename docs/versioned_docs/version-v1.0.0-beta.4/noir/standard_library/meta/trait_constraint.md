---
title: TraitConstraint
---

`std::meta::trait_constraint` contains methods on the built-in `TraitConstraint` type which represents
a trait constraint that can be used to search for a trait implementation. This is similar
syntactically to just the trait itself, but can also contain generic arguments. E.g. `Eq`, `Default`,
`BuildHasher<Poseidon2Hasher>`.

This type currently has no public methods but it can be used alongside `Type` in `implements` or `get_trait_impl`.

## Trait Implementations

```rust
impl Eq for TraitConstraint
impl Hash for TraitConstraint
```
