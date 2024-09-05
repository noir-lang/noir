---
title: StructDefinition
---

`std::meta::struct_def` contains methods on the built-in `StructDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` items in the source program.

## Methods

### as_type

#include_code as_type noir_stdlib/src/meta/struct_def.nr rust

Returns this struct as a type in the source program. If this struct has
any generics, the generics are also included as-is.

### generics

#include_code generics noir_stdlib/src/meta/struct_def.nr rust

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

#include_code fields noir_stdlib/src/meta/struct_def.nr rust

Returns each field of this struct as a pair of (field name, field type).

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
