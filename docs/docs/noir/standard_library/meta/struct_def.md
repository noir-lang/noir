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
