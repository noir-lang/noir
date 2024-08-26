---
title: FunctionDefinition
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

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

Requires the new body to be a valid expression.

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
