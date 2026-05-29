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

Because a dependency's attribute macro can call `disable` on any function it can name, an untrusted or
malicious dependency may use `disable` to silently turn off functionality in your program — for example,
disabling a contract entry point so that it can no longer be invoked. A disabled function does not fail
at compile time: it fails only when something tries to call it, producing the error message supplied to
`disable`. This makes it possible to ship a program in which an endpoint or feature has been
unintentionally taken offline, and gives opportunities for malicious DoS behaviors

Before deploying a program that pulls in third-party comptime code, you should:

- Exercise every entry point of the program (and any externally reachable function) at least once
  during testing. A `disable`d function errors loudly the first time it is called, so even a minimal
  smoke test of each endpoint is enough to surface this class of issue.
- Run `nargo expand` on any untrusted dependencies and review the output for `disable` calls on
  functions you did not expect to be disabled.

See also the [Security considerations](../../concepts/comptime.md#security-considerations) section of
the comptime documentation.

### has_named_attribute

#include_code has_named_attribute noir_stdlib/src/meta/function_def.nr rust

Returns true if this function has a custom attribute with the given name.

This matches both built-in attributes (such as `deprecated`, `export`) and
user-written attributes (tags like `#['my_tag]` and applied comptime macros).
Use `has_builtin_attribute` if you need to match only the built-in attribute
of the given name.

### has_builtin_attribute

#include_code has_builtin_attribute noir_stdlib/src/meta/function_def.nr rust

Returns true if this function has a built-in attribute with the given name.

Unlike `has_named_attribute`, this ignores user-written tag attributes and
applied comptime macros, so a user attribute that happens to share an
identifier with a built-in does not produce a false positive.

### is_unconstrained

#include_code is_unconstrained noir_stdlib/src/meta/function_def.nr rust

Returns true if this function is unconstrained.

### location

#include_code location noir_stdlib/src/meta/function_def.nr rust

Returns the source [`Location`](./location.md) where the function is defined.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the function.

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
