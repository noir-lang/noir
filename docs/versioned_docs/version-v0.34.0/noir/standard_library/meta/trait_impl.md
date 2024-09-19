---
title: TraitImpl
---

`std::meta::trait_impl` contains methods on the built-in `TraitImpl` type which represents a trait
implementation such as `impl Foo for Bar { ... }`.

## Methods

### trait_generic_args

```rust title="trait_generic_args" showLineNumbers 
comptime fn trait_generic_args(self) -> [Type] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/trait_impl.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/trait_impl.nr#L3-L5</a></sub></sup>


Returns any generic arguments on the trait of this trait implementation, if any.

```rs
impl Foo<i32, Field> for Bar { ... }

comptime {
    let bar_type = quote { Bar }.as_type();
    let foo = quote { Foo<i32, Field> }.as_trait_constraint();

    let my_impl: TraitImpl = bar_type.get_trait_impl(foo).unwrap();

    let generics = my_impl.trait_generic_args();
    assert_eq(generics.len(), 2);

    assert_eq(generics[0], quote { i32 }.as_type());
    assert_eq(generics[1], quote { Field }.as_type());
}
```

### methods

```rust title="methods" showLineNumbers 
comptime fn methods(self) -> [FunctionDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/trait_impl.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/trait_impl.nr#L8-L10</a></sub></sup>


Returns each method in this trait impl.

Example:

```rs
comptime {
    let i32_type = quote { i32 }.as_type();
    let eq = quote { Eq }.as_trait_constraint();

    let impl_eq_for_i32: TraitImpl = i32_type.get_trait_impl(eq).unwrap();
    let methods = impl_eq_for_i32.methods();

    assert_eq(methods.len(), 1);
    assert_eq(methods[0].name(), quote { eq });
}
```
