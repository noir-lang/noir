---
title: TypedExpr
---

`std::meta::typed_expr` contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.

## Methods

### get_type

```rust title="as_function_definition" showLineNumbers 
pub comptime fn as_function_definition(self) -> Option<FunctionDefinition> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typed_expr.nr#L7-L9" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typed_expr.nr#L7-L9</a></sub></sup>


If this expression refers to a function definitions, returns it. Otherwise returns `Option::none()`.

### get_type

```rust title="get_type" showLineNumbers 
pub comptime fn get_type(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typed_expr.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typed_expr.nr#L13-L15</a></sub></sup>


Returns the type of the expression, or `Option::none()` if there were errors when the expression was previously resolved.