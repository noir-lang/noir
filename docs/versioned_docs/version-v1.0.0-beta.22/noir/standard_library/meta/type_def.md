---
title: TypeDefinition
description: Inspect and transform struct/enum type definitions—fields, generics, attributes, and module context.
---

`std::meta::type_def` contains methods on the built-in `TypeDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` and `enum Name { Variant1(Fields1), ... }` items in the source program.

## Methods

### add_abi

```rust title="add_abi" showLineNumbers 
pub comptime fn add_abi(self, abi_argument: CtString) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L5-L7" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L5-L7</a></sub></sup>


Adds an abi attribute to the data type with the specified argument.

### as_type

```rust title="as_type" showLineNumbers 
pub comptime fn as_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L12-L14" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L12-L14</a></sub></sup>


Returns this type definition as a type in the source program. If this definition has
any generics, the generics are also included as-is.

### as_type_with_generics

```rust title="as_type_with_generics" showLineNumbers 
pub comptime fn as_type_with_generics(self, generics: [Type]) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L23-L25</a></sub></sup>


Returns a type from this type definition using the given generic arguments. Returns `Option::none()`
if an incorrect amount of generic arguments are given for this type.

### generics

```rust title="generics" showLineNumbers 
pub comptime fn generics(self) -> [(Type, Option<Type>)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L44-L46" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L44-L46</a></sub></sup>


Returns each generic on this type definition. Each generic is represented as a tuple containing the type,
and an optional containing the numeric type if it's a numeric generic.

Example:

```
#[example]
struct Foo<T, U, let K: u32> {
    bar: [T; K],
    baz: Baz<U, U>,
}

comptime fn example(foo: TypeDefinition) {
    assert_eq(foo.generics().len(), 3);

    // Fails because `T` isn't in scope
    // let t = quote { T }.as_type();
    // assert_eq(foo.generics()[0].0, t);
    assert(foo.generics()[0].1.is_none());

    // Last generic is numeric, so we have the numeric type available to us
    assert(foo.generics()[2].1.is_some());
}
```

### fields

```rust title="fields" showLineNumbers 
pub comptime fn fields(self, generic_args: [Type]) -> [(Quoted, Type, Quoted)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L52-L54" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L52-L54</a></sub></sup>


Returns (name, type, visibility) tuples of each field in this struct type.
Any generic types used in each field type is automatically substituted with the
provided generic arguments.

### fields_as_written

```rust title="fields_as_written" showLineNumbers 
pub comptime fn fields_as_written(self) -> [(Quoted, Type, Quoted)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L61-L63" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L61-L63</a></sub></sup>


Returns (name, type, visibility) tuples of each field in this struct type. Each type is as-is
with any generic arguments unchanged. Unless the field types are not needed,
users should generally prefer to use `TypeDefinition::fields` over this
function if possible.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L28-L30" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L28-L30</a></sub></sup>


Returns true if this type has a custom attribute with the given name.

This matches both built-in attributes and user-written attributes (tags and
applied comptime macros). Use `has_builtin_attribute` if you need to match
only the built-in attribute of the given name.

### has_builtin_attribute

```rust title="has_builtin_attribute" showLineNumbers 
pub comptime fn has_builtin_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L33-L35</a></sub></sup>


Returns true if this type has a built-in attribute with the given name.

Unlike `has_named_attribute`, this ignores user-written tag attributes and
applied comptime macros.

### location

```rust title="location" showLineNumbers 
pub comptime fn location(self) -> Location {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L37-L39" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L37-L39</a></sub></sup>


Returns the source [`Location`](./location.md) where this type is defined.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the type.

### module

```rust title="module" showLineNumbers 
pub comptime fn module(self) -> Module {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L66-L68" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L66-L68</a></sub></sup>


Returns the module where the type is defined.

### name

```rust title="name" showLineNumbers 
pub comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/type_def.nr#L71-L73" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/type_def.nr#L71-L73</a></sub></sup>


Returns the name of this type

Note that the returned quoted value will be just the type name, it will
not be the full path to the type definition, nor will it include any generics.

## Trait Implementations

```rust
impl Eq for TypeDefinition
impl Hash for TypeDefinition
```

Note that each type definition is assigned a unique ID internally and this is what is used for
equality and hashing. So even type definitions with identical generics and fields may not
be equal in this sense if they were originally different items in the source program.
