---
title: UnresolvedType
---

`std::meta::unresolved_type` contains methods on the built-in `UnresolvedType` type for the syntax of types.

## Methods

### as_mutable_reference

#include_code as_mutable_reference noir_stdlib/src/meta/unresolved_type.nr rust

If this is a mutable reference type `&mut T`, returns the mutable type `T`.

### as_slice

#include_code as_slice noir_stdlib/src/meta/unresolved_type.nr rust

If this is a slice `&[T]`, returns the element type `T`.

### is_bool

#include_code is_bool noir_stdlib/src/meta/unresolved_type.nr rust

Returns `true` if this type is `bool`.

### is_field

#include_code is_field noir_stdlib/src/meta/unresolved_type.nr rust

Returns true if this type refers to the Field type.

### is_unit

#include_code is_unit noir_stdlib/src/meta/unresolved_type.nr rust

Returns true if this type is the unit `()` type.
