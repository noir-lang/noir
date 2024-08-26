---
title: Module
---

`std::meta::module` contains methods on the built-in `Module` type which represents a module in the source program.
Note that this type represents a module generally, it isn't limited to only `mod my_submodule { ... }`
declarations in the source program.

## Methods

### name

#include_code name noir_stdlib/src/meta/module.nr rust

Returns the name of the module.

### functions

#include_code functions noir_stdlib/src/meta/module.nr rust

Returns each function in the module.

### is_contract

#include_code is_contract noir_stdlib/src/meta/module.nr rust

`true` if this module is a contract module (was declared via `contract foo { ... }`).
