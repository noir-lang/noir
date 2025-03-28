---
title: StructDefinition
---

`std::meta::struct_def` contains methods on the built-in `StructDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` items in the source program.

## Methods

### add_attribute

```rust title="add_attribute" showLineNumbers 
pub comptime fn add_attribute<let N: u32>(self, attribute: str<N>) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L3-L5</a></sub></sup>


Adds an attribute to the struct.

### add_generic

```rust title="add_generic" showLineNumbers 
pub comptime fn add_generic<let N: u32>(self, generic_name: str<N>) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L8-L10</a></sub></sup>


Adds an generic to the struct. Returns the new generic type.
Errors if the given generic name isn't a single identifier or if
the struct already has a generic with the same name.

This method should be used carefully, if there is existing code referring
to the struct type it may be checked before this function is called and
see the struct with the original number of generics. This method should
thus be preferred to use on code generated from other macros and structs
that are not used in function signatures.

Example:

```rust title="add-generic-example" showLineNumbers 
comptime fn add_generic(s: StructDefinition) {
        assert_eq(s.generics().len(), 0);
        let new_generic = s.add_generic("T");

        let generics = s.generics();
        assert_eq(generics.len(), 1);
        assert_eq(generics[0], new_generic);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_struct_definition/src/main.nr#L35-L44" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_struct_definition/src/main.nr#L35-L44</a></sub></sup>


### as_type

```rust title="as_type" showLineNumbers 
pub comptime fn as_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L15-L17" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L15-L17</a></sub></sup>


Returns this struct as a type in the source program. If this struct has
any generics, the generics are also included as-is.

### generics

```rust title="generics" showLineNumbers 
pub comptime fn generics(self) -> [Type] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L26-L28</a></sub></sup>


Returns each generic on this struct.

Example:

```
#[example]
struct Foo<T, U> {
    bar: [T; 2],
    baz: Baz<U, U>,
}

comptime fn example(foo: StructDefinition) {
    assert_eq(foo.generics().len(), 2);

    // Fails because `T` isn't in scope
    // let t = quote { T }.as_type();
    // assert_eq(foo.generics()[0], t);
}
```

### fields

```rust title="fields" showLineNumbers 
pub comptime fn fields(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L33-L35</a></sub></sup>


Returns each field of this struct as a pair of (field name, field type).

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L20-L22" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L20-L22</a></sub></sup>


Returns true if this struct has a custom attribute with the given name.

### module

```rust title="module" showLineNumbers 
pub comptime fn module(self) -> Module {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L38-L40" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L38-L40</a></sub></sup>


Returns the module where the struct is defined.

### name

```rust title="name" showLineNumbers 
pub comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L43-L45" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L43-L45</a></sub></sup>


Returns the name of this struct

Note that the returned quoted value will be just the struct name, it will
not be the full path to the struct, nor will it include any generics.

### set_fields

```rust title="set_fields" showLineNumbers 
pub comptime fn set_fields(self, new_fields: [(Quoted, Type)]) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L52-L54" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L52-L54</a></sub></sup>


Sets the fields of this struct to the given fields list where each element
is a pair of the field's name and the field's type. Expects each field name
to be a single identifier. Note that this will override any previous fields
on this struct. If those should be preserved, use `.fields()` to retrieve the
current fields on the struct type and append the new fields from there.

Example:

```rust
// Change this struct to:
// struct Foo {
//     a: u32,
//     b: i8,
// }
#[mangle_fields]
struct Foo { x: Field }

comptime fn mangle_fields(s: StructDefinition) {
    s.set_fields(&[
        (quote { a }, quote { u32 }.as_type()),
        (quote { b }, quote { i8 }.as_type()),
    ]);
}
```

## Trait Implementations

```rust
impl Eq for StructDefinition
impl Hash for StructDefinition
```

Note that each struct is assigned a unique ID internally and this is what is used for
equality and hashing. So even structs with identical generics and fields may not
be equal in this sense if they were originally different items in the source program.
