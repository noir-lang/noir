---
title: struct_def
---

# Module `std::meta::struct_def`

## `StructDefinition` methods

### add_attribute

```rust
fn add_attribute<let N: u32>(self, attribute: str<N>)
```

### add_generic

```rust
fn add_generic<let N: u32>(self, generic_name: str<N>) -> Type
```

### as_type

```rust
fn as_type(self) -> Type
```

Return a syntactic version of this struct definition as a type.
For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`

### has_named_attribute

```rust
fn has_named_attribute(self, name: Quoted) -> bool
```

### generics

```rust
fn generics(self) -> [Type]
```

Return each generic on this struct.

### fields

```rust
fn fields(self) -> [(Quoted, Type)]
```

Returns (name, type) pairs of each field in this struct. Each type is as-is
with any generic arguments unchanged.

### module

```rust
fn module(self) -> Module
```

### name

```rust
fn name(self) -> Quoted
```

### set_fields

```rust
fn set_fields(self, new_fields: [(Quoted, Type)])
```

Sets the fields of this struct to the given fields list.
All existing fields of the struct will be overridden with the given fields.
Each element of the fields list corresponds to the name and type of a field.
Each name is expected to be a single identifier.

