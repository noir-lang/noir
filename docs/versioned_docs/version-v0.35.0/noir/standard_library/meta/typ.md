---
title: Type
---

`std::meta::typ` contains methods on the built-in `Type` type used for representing
a type in the source program.

## Functions

```rust title="fresh_type_variable" showLineNumbers 
pub comptime fn fresh_type_variable() -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L5-L7" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L5-L7</a></sub></sup>


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

```rust title="serialize-setup" showLineNumbers 
trait Serialize<let N: u32> {}

impl Serialize<1> for Field {}

impl<T, let N: u32, let M: u32> Serialize<N * M> for [T; N]
    where T: Serialize<M> {}

impl<T, U, let N: u32, let M: u32> Serialize<N + M> for (T, U)
    where T: Serialize<N>, U: Serialize<M> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L20-L30" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L20-L30</a></sub></sup>

```rust title="fresh-type-variable-example" showLineNumbers 
let typevar1 = std::meta::typ::fresh_type_variable();
        let constraint = quote { Serialize<$typevar1> }.as_trait_constraint();
        let field_type = quote { Field }.as_type();

        // Search for a trait impl (binding typevar1 to 1 when the impl is found):
        assert(field_type.implements(constraint));

        // typevar1 should be bound to the "1" generic now:
        assert_eq(typevar1.as_constant().unwrap(), 1);

        // If we want to do the same with a different type, we need to
        // create a new type variable now that `typevar1` is bound
        let typevar2 = std::meta::typ::fresh_type_variable();
        let constraint = quote { Serialize<$typevar2> }.as_trait_constraint();
        let array_type = quote { [(Field, Field); 5] }.as_type();
        assert(array_type.implements(constraint));

        // Now typevar2 should be bound to the serialized pair size 2 times the array length 5
        assert_eq(typevar2.as_constant().unwrap(), 10);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L130-L150" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L130-L150</a></sub></sup>


## Methods

### as_array

```rust title="as_array" showLineNumbers 
comptime fn as_array(self) -> Option<(Type, Type)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L11-L13</a></sub></sup>


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

```rust title="as_constant" showLineNumbers 
comptime fn as_constant(self) -> Option<u32> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L16-L18" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L16-L18</a></sub></sup>


If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
return the numeric constant.

### as_integer

```rust title="as_integer" showLineNumbers 
comptime fn as_integer(self) -> Option<(bool, u8)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L21-L23" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L21-L23</a></sub></sup>


If this is an integer type, return a boolean which is `true`
if the type is signed, as well as the number of bits of this integer type.

### as_slice

```rust title="as_slice" showLineNumbers 
comptime fn as_slice(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L26-L28</a></sub></sup>


If this is a slice type, return the element type of the slice.

### as_str

```rust title="as_str" showLineNumbers 
comptime fn as_str(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L31-L33" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L31-L33</a></sub></sup>


If this is a `str<N>` type, returns the length `N` as a type.

### as_struct

```rust title="as_struct" showLineNumbers 
comptime fn as_struct(self) -> Option<(StructDefinition, [Type])> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L36-L38" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L36-L38</a></sub></sup>


If this is a struct type, returns the struct in addition to
any generic arguments on this type.

### as_tuple

```rust title="as_tuple" showLineNumbers 
comptime fn as_tuple(self) -> Option<[Type]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L41-L43" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L41-L43</a></sub></sup>


If this is a tuple type, returns each element type of the tuple.

### get_trait_impl

```rust title="get_trait_impl" showLineNumbers 
comptime fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L46-L48" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L46-L48</a></sub></sup>


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

```rust title="implements" showLineNumbers 
comptime fn implements(self, constraint: TraitConstraint) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L51-L53" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L51-L53</a></sub></sup>


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

```rust title="is_bool" showLineNumbers 
comptime fn is_bool(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L56-L58" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L56-L58</a></sub></sup>


`true` if this type is `bool`.

### is_field

```rust title="is_field" showLineNumbers 
comptime fn is_field(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L61-L63" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L61-L63</a></sub></sup>


`true` if this type is `Field`.

## Trait Implementations

```rust
impl Eq for Type
impl Hash for Type
```
Note that this is syntactic equality, this is not the same as whether two types will type check
to be the same type. Unless type inference or generics are being used however, users should not
typically have to worry about this distinction unless `std::meta::typ::fresh_type_variable` is used.
