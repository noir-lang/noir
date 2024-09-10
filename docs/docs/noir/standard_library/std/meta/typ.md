---
title: typ
---

# Module `std::meta::typ`

## `Type` methods

### as_array

```rust
fn as_array(self) -> Option<(Type, Type)>
```

### as_constant

```rust
fn as_constant(self) -> Option<u32>
```

### as_integer

```rust
fn as_integer(self) -> Option<(bool, u8)>
```

### as_slice

```rust
fn as_slice(self) -> Option<Type>
```

### as_str

```rust
fn as_str(self) -> Option<Type>
```

### as_struct

```rust
fn as_struct(self) -> Option<(StructDefinition, [Type])>
```

### as_tuple

```rust
fn as_tuple(self) -> Option<[Type]>
```

### get_trait_impl

```rust
fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl>
```

### implements

```rust
fn implements(self, constraint: TraitConstraint) -> bool
```

### is_bool

```rust
fn is_bool(self) -> bool
```

### is_field

```rust
fn is_field(self) -> bool
```

## fresh_type_variable

```rust
fn fresh_type_variable() -> Type
```

