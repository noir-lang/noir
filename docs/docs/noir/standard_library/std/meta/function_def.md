---
title: function_def
---

# Module `std::meta::function_def`

## `FunctionDefinition` methods

### add_attribute

```rust
fn add_attribute<let N: u32>(self, attribute: str<N>)
```

### body

```rust
fn body(self) -> Expr
```

### has_named_attribute

```rust
fn has_named_attribute(self, name: Quoted) -> bool
```

### is_unconstrained

```rust
fn is_unconstrained(self) -> bool
```

### module

```rust
fn module(self) -> Module
```

### name

```rust
fn name(self) -> Quoted
```

### parameters

```rust
fn parameters(self) -> [(Quoted, Type)]
```

### return_type

```rust
fn return_type(self) -> Type
```

### set_body

```rust
fn set_body(self, body: Expr)
```

### set_parameters

```rust
fn set_parameters(self, parameters: [(Quoted, Type)])
```

### set_return_type

```rust
fn set_return_type(self, return_type: Type)
```

### set_return_public

```rust
fn set_return_public(self, public: bool)
```

### set_unconstrained

```rust
fn set_unconstrained(self, value: bool)
```

