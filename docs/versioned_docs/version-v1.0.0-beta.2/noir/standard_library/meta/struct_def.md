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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L5-L7" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L5-L7</a></sub></sup>


Adds an attribute to the struct.

### add_generic

```rust title="add_generic" showLineNumbers 
pub comptime fn add_generic<let N: u32>(self, generic_name: str<N>) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L10-L12" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L10-L12</a></sub></sup>


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
        let (typ, numeric) = generics[0];
        assert_eq(typ, new_generic);
        assert(numeric.is_none());
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_struct_definition/src/main.nr#L35-L46" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_struct_definition/src/main.nr#L35-L46</a></sub></sup>


### as_type

```rust title="as_type" showLineNumbers 
pub comptime fn as_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L17-L19" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L17-L19</a></sub></sup>


Returns this struct as a type in the source program. If this struct has
any generics, the generics are also included as-is.

### generics

```rust title="generics" showLineNumbers 
pub comptime fn generics(self) -> [(Type, Option<Type>)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L29-L31" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L29-L31</a></sub></sup>


Returns each generic on this struct. Each generic is represented as a tuple containing the type, 
and an optional containing the numeric type if it's a numeric generic.

Example:

```
#[example]
struct Foo<T, U, let K: u32> {
    bar: [T; K],
    baz: Baz<U, U>,
}

comptime fn example(foo: StructDefinition) {
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
pub comptime fn fields(self, generic_args: [Type]) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L37-L39" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L37-L39</a></sub></sup>


Returns (name, type) pairs of each field in this struct.
Any generic types used in each field type is automatically substituted with the
provided generic arguments.

### fields_as_written

```rust title="fields_as_written" showLineNumbers 
pub comptime fn fields_as_written(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L46-L48" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L46-L48</a></sub></sup>


Returns (name, type) pairs of each field in this struct. Each type is as-is
with any generic arguments unchanged. Unless the field types are not needed,
users should generally prefer to use `StructDefinition::fields` over this
function if possible.

### has_named_attribute

```rust title="has_named_attribute" showLineNumbers 
pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L22-L24" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L22-L24</a></sub></sup>


Returns true if this struct has a custom attribute with the given name.

### module

```rust title="module" showLineNumbers 
pub comptime fn module(self) -> Module {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L51-L53" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L51-L53</a></sub></sup>


Returns the module where the struct is defined.

### name

```rust title="name" showLineNumbers 
pub comptime fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L56-L58" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L56-L58</a></sub></sup>


Returns the name of this struct

Note that the returned quoted value will be just the struct name, it will
not be the full path to the struct, nor will it include any generics.

### set_fields

```rust title="set_fields" showLineNumbers 
pub comptime fn set_fields(self, new_fields: [(Quoted, Type)]) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L65-L67" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L65-L67</a></sub></sup>


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
