---
title: UnresolvedType
---

`std::meta::unresolved_type` contains methods on the built-in `UnresolvedType` type for the syntax of types.

## Methods

### as_mutable_reference

```rust title="as_mutable_reference" showLineNumbers 
comptime fn as_mutable_reference(self) -> Option<UnresolvedType> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L8-L10</a></sub></sup>


If this is a mutable reference type `&mut T`, returns the mutable type `T`.

### as_slice

```rust title="as_slice" showLineNumbers 
comptime fn as_slice(self) -> Option<UnresolvedType> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L14-L16" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L14-L16</a></sub></sup>


If this is a slice `&[T]`, returns the element type `T`.

### is_bool

```rust title="is_bool" showLineNumbers 
comptime fn is_bool(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L20-L22" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L20-L22</a></sub></sup>


Returns `true` if this type is `bool`.

### is_field

```rust title="is_field" showLineNumbers 
pub comptime fn is_field(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L26-L28</a></sub></sup>


Returns true if this type refers to the Field type.

### is_unit

```rust title="is_unit" showLineNumbers 
comptime fn is_unit(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/unresolved_type.nr#L32-L34" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/unresolved_type.nr#L32-L34</a></sub></sup>


Returns true if this type is the unit `()` type.
