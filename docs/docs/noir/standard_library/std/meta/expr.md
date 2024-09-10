---
title: expr
---

# Module `std::meta::expr`

## `Expr` methods

### as_array

```rust
fn as_array(self) -> Option<[Expr]>
```

### as_assert

```rust
fn as_assert(self) -> Option<(Expr, Option<Expr>)>
```

### as_assert_eq

```rust
fn as_assert_eq(self) -> Option<(Expr, Expr, Option<Expr>)>
```

### as_assign

```rust
fn as_assign(self) -> Option<(Expr, Expr)>
```

### as_integer

```rust
fn as_integer(self) -> Option<(Field, bool)>
```

### as_binary_op

```rust
fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)>
```

### as_block

```rust
fn as_block(self) -> Option<[Expr]>
```

### as_bool

```rust
fn as_bool(self) -> Option<bool>
```

### as_cast

```rust
fn as_cast(self) -> Option<(Expr, UnresolvedType)>
```

### as_comptime

```rust
fn as_comptime(self) -> Option<[Expr]>
```

### as_function_call

```rust
fn as_function_call(self) -> Option<(Expr, [Expr])>
```

### as_if

```rust
fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)>
```

### as_index

```rust
fn as_index(self) -> Option<(Expr, Expr)>
```

### as_let

```rust
fn as_let(self) -> Option<(Expr, Option<UnresolvedType>, Expr)>
```

### as_member_access

```rust
fn as_member_access(self) -> Option<(Expr, Quoted)>
```

### as_method_call

```rust
fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])>
```

### as_repeated_element_array

```rust
fn as_repeated_element_array(self) -> Option<(Expr, Expr)>
```

### as_repeated_element_slice

```rust
fn as_repeated_element_slice(self) -> Option<(Expr, Expr)>
```

### as_slice

```rust
fn as_slice(self) -> Option<[Expr]>
```

### as_tuple

```rust
fn as_tuple(self) -> Option<[Expr]>
```

### as_unary_op

```rust
fn as_unary_op(self) -> Option<(UnaryOp, Expr)>
```

### as_unsafe

```rust
fn as_unsafe(self) -> Option<[Expr]>
```

### has_semicolon

```rust
fn has_semicolon(self) -> bool
```

### is_break

```rust
fn is_break(self) -> bool
```

### is_continue

```rust
fn is_continue(self) -> bool
```

### modify

```rust
fn modify<Env>(self, f: fn[Env](Expr) -> Option<Expr>) -> Expr
```

### quoted

```rust
fn quoted(self) -> Quoted
```

### resolve

```rust
fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr
```

