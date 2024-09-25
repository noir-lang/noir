---
title: Module
---

`std::meta::module` contains methods on the built-in `Module` type which represents a module in the source program.
Note that this type represents a module generally, it isn't limited to only `mod my_submodule { ... }`
declarations in the source program.

## Methods

### add_item

#include_code add_item noir_stdlib/src/meta/module.nr rust

Adds a top-level item (a function, a struct, a global, etc.) to the module. 
Adding multiple items in one go is also valid if the `Quoted` value has multiple items in it.  
Note that the items are type-checked as if they are inside the module they are being added to.

### functions

#include_code functions noir_stdlib/src/meta/module.nr rust

Returns each function defined in the module.

### has_named_attribute

#include_code has_named_attribute noir_stdlib/src/meta/module.nr rust

Returns true if this module has a custom attribute with the given name.

### is_contract

#include_code is_contract noir_stdlib/src/meta/module.nr rust

`true` if this module is a contract module (was declared via `contract foo { ... }`).

### name

#include_code name noir_stdlib/src/meta/module.nr rust

Returns the name of the module.

### structs

#include_code structs noir_stdlib/src/meta/module.nr rust

Returns each struct defined in the module.

## Trait Implementations

```rust
impl Eq for Module
impl Hash for Module
```

Note that each module is assigned a unique ID internally and this is what is used for
equality and hashing. So even modules with identical names and contents may not
be equal in this sense if they were originally different items in the source program.
