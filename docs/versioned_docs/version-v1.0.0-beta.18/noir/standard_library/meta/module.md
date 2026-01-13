---
title: Module
description: Work with modules in `comptime`—query names, list functions/structs, detect contracts, and add new items.
---

`std::meta::module` contains methods on the built-in `Module` type which represents a module in the source program.
Note that this type represents a module generally, it isn't limited to only `mod my_submodule { ... }`
declarations in the source program.

## Methods

### add_item

```rust title="add_item" showLineNumbers 
pub comptime fn add_item(self, item: Quoted) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L5-L7" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L5-L7</a></sub></sup>


Adds a top-level item (a function, a struct, a global, etc.) to the module.
Adding multiple items in one go is also valid if the `Quoted` value has multiple items in it.
Note that the items are type-checked as if they are inside the module they are being added to.

### child_modules

```rust title="child_modules" showLineNumbers 
pub comptime fn child_modules(self) -> [Module] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L30-L32" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L30-L32</a></sub></sup>


Returns all the child modules of the current module.

```rust title="child_modules_example" showLineNumbers 
mod my_module {
    pub mod child1 {}
    pub mod child2 {}
    pub mod child3 {
        pub mod nested_child {}
    }
}

#[test]
fn child_modules_test() {
    comptime {
        let my_module = quote [my_module].as_module().unwrap();
        let children = my_module.child_modules().map(Module::name);

        // The order children are returned in is left unspecified.
        assert_eq(children, &[quote [child1], quote [child2], quote [child3]]);
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_module/src/main.nr#L150-L169" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_module/src/main.nr#L150-L169</a></sub></sup>


### functions

```rust title="functions" showLineNumbers 
pub comptime fn functions(self) -> [FunctionDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L20-L22" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L20-L22</a></sub></sup>


Returns each function defined in the module.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L10-L12" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L10-L12</a></sub></sup>


Returns true if this module has a custom attribute with the given name.

### is_contract

```rust title="is_contract" showLineNumbers 
pub comptime fn is_contract(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L15-L17" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L15-L17</a></sub></sup>


`true` if this module is a contract module (was declared via `contract foo { ... }`).

### name

```rust title="name" showLineNumbers 
pub comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L35-L37" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L35-L37</a></sub></sup>


Returns the name of the module.
The top-level module in each crate has no name and is thus empty.

### parent

```rust title="parent" showLineNumbers 
pub comptime fn parent(self) -> Option<Module> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L40-L42" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L40-L42</a></sub></sup>


Returns the parent module of the given module, if any.

```rust title="parent_example" showLineNumbers 
mod module1 {
    pub mod module2 {}
}

#[test]
fn parent_test() {
    comptime {
        let my_module2 = quote [module1::module2].as_module().unwrap();
        assert_eq(my_module2.name(), quote [module2]);

        let my_module1 = my_module2.parent().unwrap();
        assert_eq(my_module1.name(), quote [module1]);

        // The top-level module in each crate has no name
        let top_level_module = my_module1.parent().unwrap();
        assert_eq(top_level_module.name(), quote []);
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_module/src/main.nr#L129-L148" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_module/src/main.nr#L129-L148</a></sub></sup>


### structs

```rust title="structs" showLineNumbers 
pub comptime fn structs(self) -> [TypeDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L25-L27" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L25-L27</a></sub></sup>


Returns each struct defined in the module.

## Trait Implementations

```rust
impl Eq for Module
impl Hash for Module
```

Note that each module is assigned a unique ID internally and this is what is used for
equality and hashing. So even modules with identical names and contents may not
be equal in this sense if they were originally different items in the source program.
