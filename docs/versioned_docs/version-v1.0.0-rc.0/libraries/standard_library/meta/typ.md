---
title: Type
description: Represent and analyze types at compile time—query structure, check trait bounds, and resolve trait impls.
---

`std::meta::typ` contains methods on the built-in `Type` type used for representing
a type in the source program.

## Functions

```rust title="fresh_type_variable" showLineNumbers 
pub comptime fn fresh_type_variable() -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L57-L59" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L57-L59</a></sub></sup>


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
where
    T: Serialize<M>,
{}

impl<T, U, let N: u32, let M: u32> Serialize<N + M> for (T, U)
where
    T: Serialize<N>,
    U: Serialize<M>,
{}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L14-L29" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L14-L29</a></sub></sup>

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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L137-L157" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L137-L157</a></sub></sup>


## Methods

### as_array

```rust title="as_array" showLineNumbers 
pub comptime fn as_array(self) -> Option<ArrayType> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L132-L134" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L132-L134</a></sub></sup>


If this type is an array, returns an `ArrayType`. Use `element_type()` to get the type of
each array element and `length_type()` to get the numeric-constant type describing its length.

Example:

```rust
comptime {
    let array_type = quote { [Field; 3] }.as_type();
    let array = array_type.as_array().unwrap();

    assert(array.element_type().is_field());
    assert_eq(array.length_type().as_constant().unwrap(), 3);
}
```

### as_constant

```rust title="as_constant" showLineNumbers 
pub comptime fn as_constant(self) -> Option<u32> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L146-L148" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L146-L148</a></sub></sup>


If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
return the numeric constant.

### as_integer

```rust title="as_integer" showLineNumbers 
pub comptime fn as_integer(self) -> Option<IntegerType> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L151-L153" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L151-L153</a></sub></sup>


If this is an integer type, returns an `IntegerType`. Its `is_signed()` method reports
whether the type is signed, and `bits()` returns its bit width.

### as_mutable_reference

```rust title="as_mutable_reference" showLineNumbers 
pub comptime fn as_mutable_reference(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L162-L164" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L162-L164</a></sub></sup>


If this is a mutable reference type `&mut T`, returns the mutable type `T`.

### as_vector

```rust title="as_vector" showLineNumbers 
pub comptime fn as_vector(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L168-L170" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L168-L170</a></sub></sup>


If this is a vector type, return the element type of the vector.

### as_str

```rust title="as_str" showLineNumbers 
pub comptime fn as_str(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L174-L176" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L174-L176</a></sub></sup>


If this is a `str<N>` type, returns the length `N` as a type.

### as_data_type

```rust title="as_data_type" showLineNumbers 
pub comptime fn as_data_type(self) -> Option<DataType> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L185-L187" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L185-L187</a></sub></sup>


If this is a struct or enum type, returns a `DataType`. Use `definition()` to get its
`TypeDefinition` and `generics()` to get the concrete generic arguments applied to it.

### as_tuple

```rust title="as_tuple" showLineNumbers 
pub comptime fn as_tuple(self) -> Option<[Type]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L198-L200" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L198-L200</a></sub></sup>


If this is a tuple type, returns each element type of the tuple.

### get_trait_impl

```rust title="get_trait_impl" showLineNumbers 
pub comptime fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L221-L223" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L221-L223</a></sub></sup>


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

```rust title="implements" showLineNumbers 
pub comptime fn implements(self, constraint: TraitConstraint) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L244-L246" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L244-L246</a></sub></sup>


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
pub comptime fn is_bool(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L250-L252" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L250-L252</a></sub></sup>


`true` if this type is `bool`.

### is_field

```rust title="is_field" showLineNumbers 
pub comptime fn is_field(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L256-L258" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L256-L258</a></sub></sup>


`true` if this type is `Field`.

### is_unit

```rust title="is_unit" showLineNumbers 
pub comptime fn is_unit(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L262-L264" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L262-L264</a></sub></sup>


`true` if this type is the unit `()` type.

## Trait Implementations

```rust
impl Eq for Type
impl Hash for Type
```

Note that this is syntactic equality, this is not the same as whether two types will type check
to be the same type. Unless type inference or generics are being used however, users should not
typically have to worry about this distinction unless `std::meta::typ::fresh_type_variable` is used.
