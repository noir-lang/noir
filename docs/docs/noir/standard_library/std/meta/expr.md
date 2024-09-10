---
title: expr
---

# Module `std::meta::expr`

## `Expr` methods

### as_array

```noir
fn as_array(self) -> Option<[Expr]>
```

### as_assert

```noir
fn as_assert(self) -> Option<(Expr, Option<Expr>)>
```

### as_assert_eq

```noir
fn as_assert_eq(self) -> Option<(Expr, Expr, Option<Expr>)>
```

### as_assign

```noir
fn as_assign(self) -> Option<(Expr, Expr)>
```

### as_integer

```noir
fn as_integer(self) -> Option<(Field, bool)>
```

### as_binary_op

```noir
fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)>
```

### as_block

```noir
fn as_block(self) -> Option<[Expr]>
```

### as_bool

```noir
fn as_bool(self) -> Option<bool>
```

### as_cast

```noir
fn as_cast(self) -> Option<(Expr, UnresolvedType)>
```

### as_comptime

```noir
fn as_comptime(self) -> Option<[Expr]>
```

### as_function_call

```noir
fn as_function_call(self) -> Option<(Expr, [Expr])>
```

### as_if

```noir
fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)>
```

### as_index

```noir
fn as_index(self) -> Option<(Expr, Expr)>
```

### as_let

```noir
fn as_let(self) -> Option<(Expr, Option<UnresolvedType>, Expr)>
```

### as_member_access

```noir
fn as_member_access(self) -> Option<(Expr, Quoted)>
```

### as_method_call

```noir
fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])>
```

### as_repeated_element_array

```noir
fn as_repeated_element_array(self) -> Option<(Expr, Expr)>
```

### as_repeated_element_slice

```noir
fn as_repeated_element_slice(self) -> Option<(Expr, Expr)>
```

### as_slice

```noir
fn as_slice(self) -> Option<[Expr]>
```

### as_tuple

```noir
fn as_tuple(self) -> Option<[Expr]>
```

### as_unary_op

```noir
fn as_unary_op(self) -> Option<(UnaryOp, Expr)>
```

### as_unsafe

```noir
fn as_unsafe(self) -> Option<[Expr]>
```

### has_semicolon

```noir
fn has_semicolon(self) -> bool
```

### is_break

```noir
fn is_break(self) -> bool
```

### is_continue

```noir
fn is_continue(self) -> bool
```

### modify

```noir
fn modify<Env>(self, f: fn[Env](Expr) -> Option<Expr>) -> Expr
```

### quoted

```noir
fn quoted(self) -> Quoted
```

### resolve

```noir
fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr
```

