---
title: Compile-time Code & Metaprogramming
description: Learn how to use metaprogramming in Noir to create macros or derive your own traits
keywords: [Noir, comptime, compile-time, metaprogramming, macros, quote, unquote]
sidebar_position: 15
---

## Overview

Metaprogramming in Noir is comprised of three parts:
1. `comptime` code
2. Quoting and unquoting
3. The metaprogramming API in `std::meta`

Each of these are explained in more detail in the next sections but the wide picture is that
`comptime` allows us to write code which runs at compile-time. In this `comptime` code we
can quote and unquote snippets of the program, manipulate them, and insert them in other
parts of the program. Comptime functions which do this are said to be macros. Additionally,
there's a compile-time API of built-in types and functions provided by the compiler which allows
for greater analysis and modification of programs.

---

## Comptime

`comptime` is a new keyword in Noir which marks an item as executing or existing at compile-time. It can be used in several ways:

- `comptime fn` to define functions which execute exclusively during compile-time.
- `comptime global` to define a global variable which is evaluated at compile-time.
  - Unlike runtime globals, `comptime global`s can be mutable.
- `comptime { ... }` to execute a block of statements during compile-time.
- `comptime let` to define a variable whose value is evaluated at compile-time.
- `comptime for` to run a for loop at compile-time. Syntax sugar for `comptime { for .. }`.

### Scoping

Note that while in a `comptime` context, any runtime variables _local to the current function_ are never visible.

### Evaluating

Evaluation rules of `comptime` follows the normal unconstrained evaluation rules for other Noir code. There are a few things to note though:

- Certain built-in functions may not be available, although more may be added over time.
- Evaluation order of `comptime {}` blocks within global items is currently unspecified. For example, given the following two functions we can't guarantee
which `println` will execute first. The ordering of the two printouts will be arbitrary, but should be stable across multiple compilations with the same `nargo` version as long as the program is also unchanged.

```rust
fn one() {
    comptime { println("one"); }
}

fn two() {
    comptime { println("two"); }
}
```

- Since evaluation order is unspecified, care should be taken when using mutable globals so that they do not rely on a particular ordering.
For example, using globals to generate unique ids should be fine but relying on certain ids always being produced (especially after edits to the program) should be avoided.
- Although the ordering of comptime code is usually unspecified, there are cases where it is:
  - Dependencies of a crate will always be evaluated before the dependent crate.
  - Any attributes on a function will be run before the function body is resolved. This is to allow the attribute to modify the function if necessary. Note that if the
    function itself was called at compile-time previously, it will already be resolved and cannot be modified. To prevent accidentally calling functions you wish to modify
    at compile-time, it may be helpful to sort your `comptime` annotation functions into a different submodule crate along with any dependencies they require.
  - Unlike raw `comptime {}` blocks, attributes on top-level items in the program do have a set evaluation order. Attributes within a module are evaluated top-down, and attributes
    in different modules are evaluated submodule-first. Sibling modules to the same parent module are evaluated in order of the module declarations (`mod foo; mod bar;`) in their
    parent module.

### Lowering

When a `comptime` value is used in runtime code it must be lowered into a runtime value. This means replacing the expression with the literal that it evaluated to. For example, the code:

```rust
struct Foo { array: [Field; 2], len: u32 }

fn main() {
    println(comptime {
        let mut foo = std::mem::zeroed::<Foo>();
        foo.array[0] = 4;
        foo.len = 1;
        foo
    });
}
```

will be converted to the following after `comptime` expressions are evaluated:

```rust
struct Foo { array: [Field; 2], len: u32 }

fn main() {
    println(Foo { array: [4, 0], len: 1 });
}
```

Not all types of values can be lowered. For example, references, `Type`s, and `TypeDefinition`s (among other types) cannot be lowered at all.

```rust
fn main() {
    // There's nothing we could inline here to create a Type value at runtime
    // let _ = get_type!();
}

comptime fn get_type() -> Type { ... }
```

Values of certain types may also change type when they are lowered. For example, a comptime format string will already be
formatted, and thus lowers into a runtime string instead:

```rust
fn main() {
    let foo = comptime {
        let i = 2;
        f"i = {i}"
    };
    assert_eq(foo, "i = 2");
}
```

---

## (Quasi) Quote

Macros in Noir are `comptime` functions which return code as a value which is inserted into the call site when it is lowered there.
A code value in this case is of type `Quoted` and can be created by a `quote { ... }` expression.
More specifically, the code value `quote` creates is a token stream - a representation of source code as a series of words, numbers, string literals, or operators.
For example, the expression `quote { Hi "there reader"! }` would quote three tokens: the word "hi", the string "there reader", and an exclamation mark.
You'll note that snippets that would otherwise be invalid syntax can still be quoted.

When a `Quoted` value is used in runtime code, it is lowered into a `quote { ... }` expression. Since this expression is only valid
in compile-time code however, we'd get an error if we tried this. Instead, we can use macro insertion to insert each token into the
program at that point, and parse it as an expression. To do this, we have to add a `!` after the function name returning the `Quoted` value.
If the value was created locally and there is no function returning it, `std::meta::unquote!(_)` can be used instead.
Calling such a function at compile-time without `!` will just return the `Quoted` value to be further manipulated. For example:

#include_code quote-example noir_stdlib/src/meta/mod.nr rust

For those familiar with quoting from other languages (primarily lisps), Noir's `quote` is actually a _quasiquote_.
This means we can escape the quoting by using the unquote operator to splice values in the middle of quoted code.

In addition to curly braces, you can also use square braces for the quote operator:

```rust
comptime {
    let q1 = quote { 1 };
    let q2 = quote [ 2 ];
    assert_eq(q1, q2);

    // Square braces can be used to quote mismatched curly braces if needed
    let _ = quote[}];
}
```

---

## Unquote

The unquote operator `$` is usable within a `quote` expression.
It takes a variable as an argument, evaluates the variable, and splices the resulting value into the quoted token stream at that point. For example,

```rust
comptime {
    let x = 1 + 2;
    let y = quote { $x + 4 };
}
```

The value of `y` above will be the token stream containing `3`, `+`, and `4`. We can also use this to combine `Quoted` values into larger token streams:

```rust
comptime {
    let x = quote { 1 + 2 };
    let y = quote { $x + 4 };
}
```

The value of `y` above is now the token stream containing five tokens: `1 + 2 + 4`.

Note that to unquote something, a variable name _must_ follow the `$` operator in a token stream.
If it is an expression (even a parenthesized one), it will do nothing. Most likely a parse error will be given when the macro is later unquoted.

Unquoting can also be avoided by escaping the `$` with a backslash:

```rust
comptime {
    let x = quote { 1 + 2 };

    // y contains the four tokens: `$x + 4`
    let y = quote { \$x + 4 };
}
```

### Combining Tokens

Note that `Quoted` is internally a series of separate tokens, and that all unquoting does is combine these token vectors.
This means that code which appears to append like a string actually appends like a vector internally:

```rust
comptime {
    let x = 3;
    let q = quote { foo$x }; // This is [foo, 3], not [foo3]

    // Spaces are ignored in general, they're never part of a token
    assert_eq(q, quote { foo   3 });
}
```

If you do want string semantics, you can use format strings then convert back to a `Quoted` value with `.quoted_contents()`.
Note that formatting a quoted value with multiple tokens will always insert a space between each token. If this is
undesired, you'll need to only operate on quoted values containing a single token. To do this, you can iterate
over each token of a larger quoted value with `.tokens()`:

#include_code concatenate-example noir_stdlib/src/meta/mod.nr rust

---

## Attributes

Attributes provide a way to run a `comptime` function on an item in the program.
When you use an attribute, the function with the same name will be called with that item as an argument:

```rust
#[my_struct_attribute]
struct Foo {}

comptime fn my_struct_attribute(s: StructDefinition) {
    println("Called my_struct_attribute!");
}

#[my_function_attribute]
fn foo() {}

comptime fn my_function_attribute(f: FunctionDefinition) {
    println("Called my_function_attribute!");
}
```

Anything returned from one of these functions will be inserted at top-level along with the original item.
Note that expressions are not valid at top-level so you'll get an error trying to return `3` or similar just as if you tried to write a program containing `3; struct Foo {}`.
You can insert other top-level items such as trait impls, structs, or functions this way though.
For example, this is the mechanism used to insert additional trait implementations into the program when deriving a trait impl from a struct:

#include_code derive-field-count-example noir_stdlib/src/meta/mod.nr rust

### Calling annotations with additional arguments

Arguments may optionally be given to attributes.
When this is done, these additional arguments are passed to the attribute function after the item argument.

#include_code annotation-arguments-example noir_stdlib/src/meta/mod.nr rust

We can also take any number of arguments by adding the `varargs` attribute:

#include_code annotation-varargs-example noir_stdlib/src/meta/mod.nr rust

### Attribute Evaluation Order

Unlike the evaluation order of stray `comptime {}` blocks within functions, attributes have a well-defined evaluation
order. Within a module, attributes are evaluated top to bottom. Between modules, attributes in child modules are evaluated
first. Attributes in sibling modules are resolved following the `mod foo; mod bar;` declaration order within their parent
modules.

```rust
mod foo; // attributes in foo are run first
mod bar; // followed by attributes in bar

// followed by any attributes in the parent module
#[derive(Eq)]
struct Baz {}
```

Note that because of this evaluation order, you may get an error trying to derive a trait for a struct whose fields
have not yet had the trait derived already:

```rust
// Error! `Bar` field of `Foo` does not (yet) implement Eq!
#[derive(Eq)]
struct Foo {
    bar: Bar
}

#[derive(Eq)]
struct Bar {}
```

In this case, the issue can be resolved by rearranging the structs.

---

## Comptime API

Although `comptime`, `quote`, and unquoting provide a flexible base for writing macros,
Noir's true metaprogramming ability comes from being able to interact with the compiler through a compile-time API.
This API can be accessed through built-in functions in `std::meta` as well as on methods of several `comptime` types.

The following is an incomplete list of some `comptime` types along with some useful methods on them. You can see more in the standard library [Metaprogramming section](../standard_library/meta).

- `Quoted`: A token stream
- `Type`: The type of a Noir type
  - `fn implements(self, constraint: TraitConstraint) -> bool`
    - Returns true if `self` implements the given trait constraint
- `Expr`: A syntactically valid expression. Can be used to recur on a program's parse tree to inspect how it is structured.
  - Methods:
    - `fn as_function_call(self) -> Option<(Expr, [Expr])>`
      - If this is a function call expression, return `(function, arguments)`
    - `fn as_block(self) -> Option<[Expr]>`
      - If this is a block, return each statement in the block
- `FunctionDefinition`: A function definition
  - Methods:
    - `fn parameters(self) -> [(Quoted, Type)]`
      - Returns a slice of `(name, type)` pairs for each parameter
- `StructDefinition`: A struct definition
  - Methods:
    - `fn as_type(self) -> Type`
      - Returns this `StructDefinition` as a `Type`. Any generics are kept as-is
    - `fn generics(self) -> [Quoted]`
      - Return the name of each generic on this struct
    - `fn fields(self) -> [(Quoted, Type)]`
      - Return the name and type of each field
- `TraitConstraint`: A trait constraint such as `From<Field>`
- `TypedExpr`: A type-checked expression.
- `UnresolvedType`: A syntactic notation that refers to a Noir type that hasn't been resolved yet

There are many more functions available by exploring the `std::meta` module and its submodules.
Using these methods is the key to writing powerful metaprogramming libraries.

### `#[use_callers_scope]`

Since certain functions such as `Quoted::as_type`, `Expression::as_type`, or `Quoted::as_trait_constraint` will attempt
to resolve their contents in a particular scope - it can be useful to change the scope they resolve in. By default
these functions will resolve in the current function's scope which is usually the attribute function they are called in.
If you're working on a library however, this may be a completely different module or crate to the item you're trying to
use the attribute on. If you want to be able to use `Quoted::as_type` to refer to types local to the caller's scope for
example, you can annotate your attribute function with `#[use_callers_scope]`. This will ensure your attribute, and any
closures it uses, can refer to anything in the caller's scope. `#[use_callers_scope]` also works recursively. So if both
your attribute function and a helper function it calls use it, then they can both refer to the same original caller.

---

## Example: Derive

Using all of the above, we can write a `derive` macro that behaves similarly to Rust's but is not built into the language.
From the user's perspective it will look like this:

```rust
// Example usage
#[derive(Default, Eq, Ord)]
struct MyStruct { my_field: u32 }
```

To implement `derive` we'll have to create a `comptime` function that accepts
a variable amount of traits.

#include_code derive_example noir_stdlib/src/meta/mod.nr rust

Registering a derive function could be done as follows:

#include_code derive_via noir_stdlib/src/meta/mod.nr rust

#include_code big-derive-usage-example noir_stdlib/src/meta/mod.nr rust
