---
title: FunctionDefinition
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

### add_attribute

```rust title="add_attribute" showLineNumbers 
comptime fn add_attribute<let N: u32>(self, attribute: str<N>) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L3-L5</a></sub></sup>


Adds an attribute to the function. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### body

```rust title="body" showLineNumbers 
comptime fn body(self) -> Expr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L8-L10</a></sub></sup>


Returns the body of the function as an expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L13-L15</a></sub></sup>


Returns true if this function has a custom attribute with the given name.

### is_unconstrained

```rust title="is_unconstrained" showLineNumbers 
comptime fn is_unconstrained(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L18-L20</a></sub></sup>


Returns true if this function is unconstrained.

### module

```rust title="module" showLineNumbers 
comptime fn module(self) -> Module {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L23-L25</a></sub></sup>


Returns the module where the function is defined.

### name

```rust title="name" showLineNumbers 
comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L28-L30" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L28-L30</a></sub></sup>


Returns the name of the function.

### parameters

```rust title="parameters" showLineNumbers 
comptime fn parameters(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L33-L35</a></sub></sup>


Returns each parameter of the function as a tuple of (parameter pattern, parameter type).

### return_type

```rust title="return_type" showLineNumbers 
comptime fn return_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L38-L40" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L38-L40</a></sub></sup>


The return type of the function.

### set_body

```rust title="set_body" showLineNumbers 
comptime fn set_body(self, body: Expr) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L43-L45" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L43-L45</a></sub></sup>


Mutate the function body to a new expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_parameters

```rust title="set_parameters" showLineNumbers 
comptime fn set_parameters(self, parameters: [(Quoted, Type)]) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L48-L50" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L48-L50</a></sub></sup>


Mutates the function's parameters to a new set of parameters. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

Expects a slice of (parameter pattern, parameter type) for each parameter. Requires
each parameter pattern to be a syntactically valid parameter.

### set_return_type

```rust title="set_return_type" showLineNumbers 
comptime fn set_return_type(self, return_type: Type) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L53-L55" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L53-L55</a></sub></sup>


Mutates the function's return type to a new type. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_return_public

```rust title="set_return_public" showLineNumbers 
comptime fn set_return_public(self, public: bool) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L58-L60" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L58-L60</a></sub></sup>


Mutates the function's return visibility to public (if `true` is given) or private (if `false` is given).
This is only valid on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_unconstrained

```rust title="set_unconstrained" showLineNumbers 
comptime fn set_unconstrained(self, value: bool) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L63-L65" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L63-L65</a></sub></sup>


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
