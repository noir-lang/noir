---
title: TypedExpr
---

`std::meta::typed_expr` contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.

## Methods

### as_function_definition

#include_code as_function_definition noir_stdlib/src/meta/typed_expr.nr rust

If this expression refers to a function definitions, returns it. Otherwise returns `Option::none()`.