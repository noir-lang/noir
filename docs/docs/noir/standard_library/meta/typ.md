---
title: Type
---

`std::meta::typ` contains methods on the built-in `Type` type used for representing
a type in the source program.

## Functions

#include_code fresh_type_variable noir_stdlib/src/meta/typ.nr rust

Creates and returns an unbound type variable. This is a special kind of type internal
to type checking which will type check with any other type. When it is type checked
against another type it will also be set to that type. For example, if `a` is a type
variable and we have the type equality `(a, i32) = (u8, i32)`, the compiler will set
`a` equal to `u8`.

Unbound type variables will often be rendered as `_` while printing them. Bound type
variables will appear as the type they are bound to.

This can be used in conjunction with functions which internally perform type checks
such as `Type::implements` or `Type::get_trait_impl` to potentially grab some of the types used.

Note that calling `Type::implements` or `Type::get_trait_impl` on a type variable will always
fail.

Example:

#include_code serialize-setup test_programs/compile_success_empty/comptime_type/src/main.nr rust
#include_code fresh-type-variable-example test_programs/compile_success_empty/comptime_type/src/main.nr rust

## Methods

### as_array

#include_code as_array noir_stdlib/src/meta/typ.nr rust

If this type is an array, return a pair of (element type, size type).

Example:

```rust
comptime {
    let array_type = quote { [Field; 3] }.as_type();
    let (field_type, three_type) = array_type.as_array().unwrap();

    assert(field_type.is_field());
    assert_eq(three_type.as_constant().unwrap(), 3);
}
```

### as_constant

#include_code as_constant noir_stdlib/src/meta/typ.nr rust

If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
return the numeric constant.

### as_integer

#include_code as_integer noir_stdlib/src/meta/typ.nr rust

If this is an integer type, return a boolean which is `true`
if the type is signed, as well as the number of bits of this integer type.

### as_mutable_reference

#include_code as_mutable_reference noir_stdlib/src/meta/typ.nr rust

If this is a mutable reference type `&mut T`, returns the mutable type `T`.

### as_slice

#include_code as_slice noir_stdlib/src/meta/typ.nr rust

If this is a slice type, return the element type of the slice.

### as_str

#include_code as_str noir_stdlib/src/meta/typ.nr rust

If this is a `str<N>` type, returns the length `N` as a type.

### as_struct

#include_code as_struct noir_stdlib/src/meta/typ.nr rust

If this is a struct type, returns the struct in addition to
any generic arguments on this type.

### as_tuple

#include_code as_tuple noir_stdlib/src/meta/typ.nr rust

If this is a tuple type, returns each element type of the tuple.

### get_trait_impl

#include_code get_trait_impl noir_stdlib/src/meta/typ.nr rust

Retrieves the trait implementation that implements the given
trait constraint for this type. If the trait constraint is not
found, `None` is returned. Note that since the concrete trait implementation
for a trait constraint specified in a `where` clause is unknown,
this function will return `None` in these cases. If you only want to know
whether a type implements a trait, use `implements` instead.

Example:

```rust
comptime {
    let field_type = quote { Field }.as_type();
    let default = quote { Default }.as_trait_constraint();

    let the_impl: TraitImpl = field_type.get_trait_impl(default).unwrap();
    assert(the_impl.methods().len(), 1);
}
```

### implements

#include_code implements noir_stdlib/src/meta/typ.nr rust

`true` if this type implements the given trait. Note that unlike
`get_trait_impl` this will also return true for any `where` constraints
in scope.

Example:

```rust
fn foo<T>() where T: Default {
    comptime {
        let field_type = quote { Field }.as_type();
        let default = quote { Default }.as_trait_constraint();
        assert(field_type.implements(default));

        let t = quote { T }.as_type();
        assert(t.implements(default));
    }
}
```

### is_bool

#include_code is_bool noir_stdlib/src/meta/typ.nr rust

`true` if this type is `bool`.

### is_field

#include_code is_field noir_stdlib/src/meta/typ.nr rust

`true` if this type is `Field`.

### is_unit

#include_code is_unit noir_stdlib/src/meta/typ.nr rust

`true` if this type is the unit `()` type.

## Trait Implementations

```rust
impl Eq for Type
impl Hash for Type
```
Note that this is syntactic equality, this is not the same as whether two types will type check
to be the same type. Unless type inference or generics are being used however, users should not
typically have to worry about this distinction unless `std::meta::typ::fresh_type_variable` is used.
