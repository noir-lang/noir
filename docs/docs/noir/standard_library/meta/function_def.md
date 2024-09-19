---
title: FunctionDefinition
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

### add_attribute

#include_code add_attribute noir_stdlib/src/meta/function_def.nr rust

Adds an attribute to the function. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### body

#include_code body noir_stdlib/src/meta/function_def.nr rust

Returns the body of the function as an expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

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

### set_body

#include_code set_body noir_stdlib/src/meta/function_def.nr rust

Mutate the function body to a new expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_parameters

#include_code set_parameters noir_stdlib/src/meta/function_def.nr rust

Mutates the function's parameters to a new set of parameters. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

Expects a slice of (parameter pattern, parameter type) for each parameter. Requires
each parameter pattern to be a syntactically valid parameter.

### set_return_type

#include_code set_return_type noir_stdlib/src/meta/function_def.nr rust

Mutates the function's return type to a new type. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_return_public

#include_code set_return_public noir_stdlib/src/meta/function_def.nr rust

Mutates the function's return visibility to public (if `true` is given) or private (if `false` is given).
This is only valid on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_unconstrained

#include_code set_unconstrained noir_stdlib/src/meta/function_def.nr rust

Mutates the function to be unconstrained (if `true` is given) or not (if `false` is given).
This is only valid on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

## Trait Implementations

```rust
impl Eq for FunctionDefinition
impl Hash for FunctionDefinition
```

Note that each function is assigned a unique ID internally and this is what is used for
equality and hashing. So even functions with identical signatures and bodies may not
be equal in this sense if they were originally different items in the source program.
