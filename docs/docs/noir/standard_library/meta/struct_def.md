---
title: StructDefinition
---

`std::meta::struct_def` contains methods on the built-in `StructDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` items in the source program.

## Methods

### add_attribute

#include_code add_attribute noir_stdlib/src/meta/struct_def.nr rust

Adds an attribute to the struct.

### add_generic

#include_code add_generic noir_stdlib/src/meta/struct_def.nr rust

Adds an generic to the struct. Returns the new generic type.
Errors if the given generic name isn't a single identifier or if
the struct already has a generic with the same name.

This method should be used carefully, if there is existing code referring
to the struct type it may be checked before this function is called and
see the struct with the original number of generics. This method should
thus be preferred to use on code generated from other macros and structs
that are not used in function signatures.

Example:

#include_code add-generic-example test_programs/compile_success_empty/comptime_struct_definition/src/main.nr rust

### as_type

#include_code as_type noir_stdlib/src/meta/struct_def.nr rust

Returns this struct as a type in the source program. If this struct has
any generics, the generics are also included as-is.

### generics

#include_code generics noir_stdlib/src/meta/struct_def.nr rust

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

#include_code fields noir_stdlib/src/meta/struct_def.nr rust

Returns (name, type) pairs of each field in this struct.
Any generic types used in each field type is automatically substituted with the
provided generic arguments.

### fields_as_written

#include_code fields_as_written noir_stdlib/src/meta/struct_def.nr rust

Returns (name, type) pairs of each field in this struct. Each type is as-is
with any generic arguments unchanged. Unless the field types are not needed,
users should generally prefer to use `StructDefinition::fields` over this
function if possible.

### has_named_attribute

#include_code has_named_attribute noir_stdlib/src/meta/struct_def.nr rust

Returns true if this struct has a custom attribute with the given name.

### module

#include_code module noir_stdlib/src/meta/struct_def.nr rust

Returns the module where the struct is defined.

### name

#include_code name noir_stdlib/src/meta/struct_def.nr rust

Returns the name of this struct

Note that the returned quoted value will be just the struct name, it will
not be the full path to the struct, nor will it include any generics.

### set_fields

#include_code set_fields noir_stdlib/src/meta/struct_def.nr rust

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
