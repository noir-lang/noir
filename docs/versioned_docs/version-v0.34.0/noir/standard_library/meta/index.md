---
title: Metaprogramming
description: Noir's Metaprogramming API
keywords: [metaprogramming, comptime, macros, macro, quote, unquote]
---

`std::meta` is the entry point for Noir's metaprogramming API. This consists of `comptime` functions
and types used for inspecting and modifying Noir programs.

## Functions

### type_of

```rust title="type_of" showLineNumbers 
pub comptime fn type_of<T>(x: T) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/mod.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/mod.nr#L26-L28</a></sub></sup>


Returns the type of a variable at compile-time.

Example:
```rust
comptime {
    let x: i32 = 1;
    let x_type: Type = std::meta::type_of(x);

    assert_eq(x_type, quote { i32 }.as_type());
}
```

### unquote

```rust title="unquote" showLineNumbers 
pub comptime fn unquote(code: Quoted) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/mod.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/mod.nr#L18-L20</a></sub></sup>


Unquotes the passed-in token stream where this function was called.

Example:
```rust
comptime {
    let code = quote { 1 + 2 };

    // let x = 1 + 2;
    let x = unquote!(code);
}
```

### derive

```rust title="derive" showLineNumbers 
#[varargs]
pub comptime fn derive(s: StructDefinition, traits: [TraitDefinition]) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/mod.nr#L46-L49" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/mod.nr#L46-L49</a></sub></sup>


Attribute placed on struct definitions.

Creates a trait impl for each trait passed in as an argument.
To do this, the trait must have a derive handler registered
with `derive_via` beforehand. The traits in the stdlib that
can be derived this way are `Eq`, `Ord`, `Default`, and `Hash`.

Example:
```rust
#[derive(Eq, Default)]
struct Foo<T> {
    x: i32,
    y: T,
}

fn main() {
    let foo1 = Foo::default();
    let foo2 = Foo { x: 0, y: &[0] };
    assert_eq(foo1, foo2);
}
```

### derive_via

```rust title="derive_via_signature" showLineNumbers 
pub comptime fn derive_via(t: TraitDefinition, f: DeriveFunction) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/mod.nr#L68-L70" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/mod.nr#L68-L70</a></sub></sup>


Attribute placed on trait definitions.

Registers a function to create impls for the given trait
when the trait is used in a `derive` call. Users may use
this to register their own functions to enable their traits
to be derived by `derive`.

Because this function requires a function as an argument which
should produce a trait impl for any given struct, users may find
it helpful to use a function like `std::meta::make_trait_impl` to
help creating these impls.

Example:
```rust
#[derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

comptime fn derive_do_nothing(s: StructDefinition) -> Quoted {
    let typ = s.as_type();
    quote {
        impl DoNothing for $typ {
            fn do_nothing(self) {
                println("Nothing");
            }
        }
    }
}
```

As another example, `derive_eq` in the stdlib is used to derive the `Eq`
trait for any struct. It makes use of `make_trait_impl` to do this:

```rust title="derive_eq" showLineNumbers 
comptime fn derive_eq(s: StructDefinition) -> Quoted {
    let signature = quote { fn eq(_self: Self, _other: Self) -> bool };
    let for_each_field = |name| quote { (_self.$name == _other.$name) };
    let body = |fields| {
        if s.fields().len() == 0 {
            quote { true }
        } else {
            fields
        }
    };
    crate::meta::make_trait_impl(s, quote { Eq }, signature, for_each_field, quote { & }, body)
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/cmp.nr#L10-L23" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/cmp.nr#L10-L23</a></sub></sup>


### make_trait_impl

```rust title="make_trait_impl" showLineNumbers 
pub comptime fn make_trait_impl<Env1, Env2>(
    s: StructDefinition,
    trait_name: Quoted,
    function_signature: Quoted,
    for_each_field: fn[Env1](Quoted) -> Quoted,
    join_fields_with: Quoted,
    body: fn[Env2](Quoted) -> Quoted
) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/mod.nr#L87-L96" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/mod.nr#L87-L96</a></sub></sup>


A helper function to more easily create trait impls while deriving traits.

Note that this function only works for traits which:
1. Have only one method
2. Have no generics on the trait itself.
  - E.g. Using this on a trait such as `trait Foo<T> { ... }` will result in the
    generated impl incorrectly missing the `T` generic.

If your trait fits these criteria then `make_trait_impl` is likely the easiest
way to write your derive handler. The arguments are as follows:

- `s`: The struct to make the impl for
- `trait_name`: The name of the trait to derive. E.g. `quote { Eq }`.
- `function_signature`: The signature of the trait method to derive. E.g. `fn eq(self, other: Self) -> bool`.
- `for_each_field`: An operation to be performed on each field. E.g. `|name| quote { (self.$name == other.$name) }`.
- `join_fields_with`: A separator to join each result of `for_each_field` with.
  E.g. `quote { & }`. You can also use an empty `quote {}` for no separator.
- `body`: The result of the field operations are passed into this function for any final processing.
  This is the place to insert any setup/teardown code the trait requires. If the trait doesn't require
  any such code, you can return the body as-is: `|body| body`.

Example deriving `Hash`:

```rust title="derive_hash" showLineNumbers 
comptime fn derive_hash(s: StructDefinition) -> Quoted {
    let name = quote { Hash };
    let signature = quote { fn hash<H>(_self: Self, _state: &mut H) where H: std::hash::Hasher };
    let for_each_field = |name| quote { _self.$name.hash(_state); };
    crate::meta::make_trait_impl(s, name, signature, for_each_field, quote {}, |fields| fields)
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/hash/mod.nr#L147-L154" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/hash/mod.nr#L147-L154</a></sub></sup>


Example deriving `Ord`:

```rust title="derive_ord" showLineNumbers 
comptime fn derive_ord(s: StructDefinition) -> Quoted {
    let signature = quote { fn cmp(_self: Self, _other: Self) -> std::cmp::Ordering };
    let for_each_field = |name| quote {
        if result == std::cmp::Ordering::equal() {
            result = _self.$name.cmp(_other.$name);
        }
    };
    let body = |fields| quote {
        let mut result = std::cmp::Ordering::equal();
        $fields
        result
    };
    crate::meta::make_trait_impl(s, quote { Ord }, signature, for_each_field, quote {}, body)
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/cmp.nr#L181-L196" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/cmp.nr#L181-L196</a></sub></sup>

