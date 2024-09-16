---
title: Module
---

`std::meta::module` contains methods on the built-in `Module` type which represents a module in the source program.
Note that this type represents a module generally, it isn't limited to only `mod my_submodule { ... }`
declarations in the source program.

## Methods

### add_item

```rust title="add_item" showLineNumbers 
comptime fn add_item(self, item: Quoted) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L3-L5</a></sub></sup>


Adds a top-level item (a function, a struct, a global, etc.) to the module. 
Adding multiple items in one go is also valid if the `Quoted` value has multiple items in it.  
Note that the items are type-checked as if they are inside the module they are being added to.

### functions

```rust title="functions" showLineNumbers 
comptime fn functions(self) -> [FunctionDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L18-L20</a></sub></sup>


Returns each function defined in the module.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L8-L10</a></sub></sup>


Returns true if this module has a custom attribute with the given name.

### is_contract

```rust title="is_contract" showLineNumbers 
comptime fn is_contract(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L13-L15</a></sub></sup>


`true` if this module is a contract module (was declared via `contract foo { ... }`).

### name

```rust title="name" showLineNumbers 
comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L28-L30" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L28-L30</a></sub></sup>


Returns the name of the module.

### structs

```rust title="structs" showLineNumbers 
comptime fn structs(self) -> [StructDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L23-L25</a></sub></sup>


Returns each struct defined in the module.

## Trait Implementations

```rust
impl Eq for Module
impl Hash for Module
```

Note that each module is assigned a unique ID internally and this is what is used for
equality and hashing. So even modules with identical names and contents may not
be equal in this sense if they were originally different items in the source program.
