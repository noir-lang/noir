---
title: FunctionDefinition
description: Inspect and mutate function definitions in `comptime`—read signatures, body, attributes, and adjust parameters or return types.
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

### as_typed_expr

#include_code as_typed_expr noir_stdlib/src/meta/function_def.nr rust

Returns this function as a `TypedExpr`, which can be unquoted. For example:

```rust
let typed_expr = some_function.as_typed_expr();
let _ = quote { $typed_expr(1, 2, 3); };
```

### body

#include_code body noir_stdlib/src/meta/function_def.nr rust

Returns the body of the function as an expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### disable

#include_code disable noir_stdlib/src/meta/function_def.nr rust

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

#include_code has_named_attribute noir_stdlib/src/meta/function_def.nr rust

Returns true if this function has a custom attribute with the given name.

### is_unconstrained

#include_code is_unconstrained noir_stdlib/src/meta/function_def.nr rust

Returns true if this function is unconstrained.

### module

#include_code module noir_stdlib/src/meta/function_def.nr rust

Returns the module where the function is defined.

### name

#include_code name noir_stdlib/src/meta/function_def.nr rust

Returns the name of the function.

### parameters

#include_code parameters noir_stdlib/src/meta/function_def.nr rust

Returns each parameter of the function as a tuple of (parameter pattern, parameter type).

### return_type

#include_code return_type noir_stdlib/src/meta/function_def.nr rust

The return type of the function.

### visibility

#include_code visibility noir_stdlib/src/meta/function_def.nr rust

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
