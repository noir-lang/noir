---
title: TypeDefinition
description: Inspect and transform struct/enum type definitions—fields, generics, attributes, and module context.
---

`std::meta::type_def` contains methods on the built-in `TypeDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` and `enum Name { Variant1(Fields1), ... }` items in the source program.

## Methods

### add_abi

#include_code add_abi noir_stdlib/src/meta/type_def.nr rust

Adds an abi attribute to the data type with the specified argument.

### as_type

#include_code as_type noir_stdlib/src/meta/type_def.nr rust

Returns this type definition as a type in the source program. If this definition has
any generics, the generics are also included as-is.

### as_type_with_generics

#include_code as_type_with_generics noir_stdlib/src/meta/type_def.nr rust

Returns a type from this type definition using the given generic arguments. Returns `Option::none()`
if an incorrect amount of generic arguments are given for this type.

### generics

#include_code generics noir_stdlib/src/meta/type_def.nr rust

Returns each generic on this type definition as a `TypeGeneric`. Use `typ()` to get the
generic's type variable. For a numeric generic, `numeric_type()` contains its numeric type;
for other generics, it returns `None`.

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
    // assert_eq(foo.generics()[0].typ(), t);
    assert(foo.generics()[0].numeric_type().is_none());

    // Last generic is numeric, so we have the numeric type available to us
    assert(foo.generics()[2].numeric_type().is_some());
}
```

### fields

#include_code fields noir_stdlib/src/meta/type_def.nr rust

Returns each field in this struct type as a `StructField`. Use `name()` and `typ()` to inspect
its name and type. `visibility()` returns the field's [`ItemVisibility`](./item_visibility.md).
Generic types used in each field type are automatically substituted with the provided generic
arguments.

### fields_as_written

#include_code fields_as_written noir_stdlib/src/meta/type_def.nr rust

Returns each field in this struct type as a `StructField`. Use `name()` and `typ()` to inspect
its name and type. `visibility()` returns the field's [`ItemVisibility`](./item_visibility.md).
Each type is returned as written, with any generic arguments unchanged. Unless the field types
are not needed, users should generally prefer to use `TypeDefinition::fields` over this function
if possible.

### has_named_attribute

#include_code has_named_attribute noir_stdlib/src/meta/type_def.nr rust

Returns true if this type has a custom attribute with the given name.

This matches both built-in attributes and user-written attributes (tags and
applied comptime macros). Use `has_builtin_attribute` if you need to match
only the built-in attribute of the given name.

### named_attribute_args

#include_code named_attribute_args noir_stdlib/src/meta/type_def.nr rust

Returns the arguments of each occurrence of the attribute with the given name, as token
streams. The outer slice has one entry per occurrence — empty when the attribute is absent,
so this is a superset of `has_named_attribute` — and each inner slice holds that occurrence's
argument expressions, which can be spliced into generated code with `quote`.

### has_builtin_attribute

#include_code has_builtin_attribute noir_stdlib/src/meta/type_def.nr rust

Returns true if this type has a built-in attribute with the given name.

Unlike `has_named_attribute`, this ignores user-written tag attributes and
applied comptime macros.

### location

#include_code location noir_stdlib/src/meta/type_def.nr rust

Returns the source [`Location`](./location.md) where this type is defined.
This can be passed to `std::meta::error` or `std::meta::warn` to attach a diagnostic to the type.

### module

#include_code module noir_stdlib/src/meta/type_def.nr rust

Returns the module where the type is defined.

### name

#include_code name noir_stdlib/src/meta/type_def.nr rust

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
