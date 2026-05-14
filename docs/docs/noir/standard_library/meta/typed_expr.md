---
title: TypedExpr
description: Resolved, type-checked expressions—retrieve types, access referenced function definitions, and more.
---

`std::meta::typed_expr` contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.

## Methods

### as_function_definition

#include_code as_function_definition noir_stdlib/src/meta/typed_expr.nr rust

If this expression refers to a function definition, returns it. Otherwise returns `Option::none()`.

### get_type

#include_code get_type noir_stdlib/src/meta/typed_expr.nr rust

Returns the type of the expression, or `Option::none()` if there were errors when the expression was previously resolved.

### location

#include_code location noir_stdlib/src/meta/typed_expr.nr rust

Returns the source [`Location`](./location.md) of this expression.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the expression.