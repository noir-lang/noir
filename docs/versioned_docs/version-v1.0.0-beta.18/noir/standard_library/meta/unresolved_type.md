---
title: UnresolvedType
description: Work with the syntactic form of types—inspect references, vectors, and primitive kind checks before resolution.
---

`std::meta::unresolved_type` contains methods on the built-in `UnresolvedType` type for the syntax of types.

## Methods

### as_mutable_reference

```rust title="as_mutable_reference" showLineNumbers 
pub comptime fn as_mutable_reference(self) -> Option<UnresolvedType> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L8-L10</a></sub></sup>


If this is a mutable reference type `&mut T`, returns the mutable type `T`.

### as_vector

```rust title="as_vector" showLineNumbers 
pub comptime fn as_vector(self) -> Option<UnresolvedType> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L14-L16" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L14-L16</a></sub></sup>


If this is a vector `&[T]`, returns the element type `T`.

### is_bool

```rust title="is_bool" showLineNumbers 
pub comptime fn is_bool(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L25-L27" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L25-L27</a></sub></sup>


Returns `true` if this type is `bool`.

### is_field

```rust title="is_field" showLineNumbers 
pub comptime fn is_field(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L31-L33" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L31-L33</a></sub></sup>


Returns true if this type refers to the Field type.

### is_unit

```rust title="is_unit" showLineNumbers 
pub comptime fn is_unit(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L37-L39" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L37-L39</a></sub></sup>


Returns true if this type is the unit `()` type.
