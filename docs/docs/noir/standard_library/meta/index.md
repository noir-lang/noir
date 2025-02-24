---
title: Metaprogramming
description: Noir's Metaprogramming API
keywords: [metaprogramming, comptime, macros, macro, quote, unquote]
---

`std::meta` is the entry point for Noir's metaprogramming API. This consists of `comptime` functions
and types used for inspecting and modifying Noir programs.

## Functions

### type_of

#include_code type_of noir_stdlib/src/meta/mod.nr rust

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

#include_code unquote noir_stdlib/src/meta/mod.nr rust

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

#include_code derive noir_stdlib/src/meta/mod.nr rust

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

#include_code derive_via_signature noir_stdlib/src/meta/mod.nr rust

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

#include_code derive_eq noir_stdlib/src/cmp.nr rust

### make_trait_impl

#include_code make_trait_impl noir_stdlib/src/meta/mod.nr rust

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
- `body`: The result of the field operations is passed into this function for any final processing.
  This is the place to insert any setup/teardown code the trait requires. If the trait doesn't require
  any such code, you can return the body as-is: `|body| body`.

Example deriving `Hash`:

#include_code derive_hash noir_stdlib/src/hash/mod.nr rust

Example deriving `Ord`:

#include_code derive_ord noir_stdlib/src/cmp.nr rust
