---
title: Type
---

`std::meta::typ` contains methods on the built-in `Type` type used for representing
a type in the source program.

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

### as_slice

#include_code as_slice noir_stdlib/src/meta/typ.nr rust

If this is a slice type, return the element type of the slice.

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
for a trait constraint specified from a `where` clause is unknown,
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

## Trait Implementations

```rust
impl Eq for Type
```
Note that this is syntactic equality, this is not the same as whether two types will type check
to be the same type. Unless type inference or generics are being used however, users should not
typically have to worry about this distinction.
