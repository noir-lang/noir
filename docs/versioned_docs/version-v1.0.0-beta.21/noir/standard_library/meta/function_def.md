---
title: FunctionDefinition
description: Inspect and mutate function definitions in `comptime`—read signatures, body, attributes, and adjust parameters or return types.
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

### as_typed_expr

```rust title="as_typed_expr" showLineNumbers 
pub comptime fn as_typed_expr(self) -> TypedExpr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L3-L5</a></sub></sup>


Returns this function as a `TypedExpr`, which can be unquoted. For example:

```rust
let typed_expr = some_function.as_typed_expr();
let _ = quote { $typed_expr(1, 2, 3); };
```

### body

```rust title="body" showLineNumbers 
pub comptime fn body(self) -> Expr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L8-L10</a></sub></sup>


Returns the body of the function as an expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### disable

```rust title="disable" showLineNumbers 
pub comptime fn disable(self, error_message: CtString) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L13-L15</a></sub></sup>


Disables calling the given function, issuing an error with the provided message if it is ever called.

If an attribute generates a new version of a function that is meant to be called instead, calling `disable`
on the old function can be helpful to point a user to the new function to call.

If the disabled function is part of a contract, this function will also remove it from the contract
interface as if `#[contract_library_method]` was used.

This method requires the given function to yet be elaborated by the compiler - otherwise it is possible
it is already called elsewhere without an error.

#### Security

Before switching over to use these newly generated functions, users should run `nargo expand` on any
untrusted dependencies to ensure their bodies are as expected.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L18-L20</a></sub></sup>


Returns true if this function has a custom attribute with the given name.

### is_unconstrained

```rust title="is_unconstrained" showLineNumbers 
pub comptime fn is_unconstrained(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L23-L25</a></sub></sup>


Returns true if this function is unconstrained.

### module

```rust title="module" showLineNumbers 
pub comptime fn module(self) -> Module {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L28-L30" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L28-L30</a></sub></sup>


Returns the module where the function is defined.

### name

```rust title="name" showLineNumbers 
pub comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L33-L35</a></sub></sup>


Returns the name of the function.

### parameters

```rust title="parameters" showLineNumbers 
pub comptime fn parameters(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L38-L40" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L38-L40</a></sub></sup>


Returns each parameter of the function as a tuple of (parameter pattern, parameter type).

### return_type

```rust title="return_type" showLineNumbers 
pub comptime fn return_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L43-L45" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L43-L45</a></sub></sup>


The return type of the function.

### visibility

```rust title="visibility" showLineNumbers 
pub comptime fn visibility(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L48-L50" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L48-L50</a></sub></sup>


Returns the function's visibility as a `Quoted` value, which will be one of:
- `quote { }`: the function is private
- `quote { pub }`: the function is `pub`
- `quote { pub(crate) }`: the function is `pub(crate)`

## Trait Implementations

```rust
impl Eq for FunctionDefinition
impl Hash for FunctionDefinition
```

Note that each function is assigned a unique ID internally and this is what is used for
equality and hashing. So even functions with identical signatures and bodies may not
be equal in this sense if they were originally different items in the source program.
