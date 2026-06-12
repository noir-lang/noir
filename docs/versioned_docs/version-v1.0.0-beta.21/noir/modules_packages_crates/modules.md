---
title: Modules
description:
  Learn how to organize your files using modules in Noir, following the same convention as Rust's
  module system. Examples included.
keywords: [Noir, Rust, modules, organizing files, sub-modules]
sidebar_position: 2
---

Noir's module system follows the same convention as the _newer_ version of Rust's module system.

## Purpose of Modules

Modules are used to organize files. Without modules all of your code would need to live in a single
file. In Noir, the compiler does not automatically scan all of your files to detect modules. This
must be done explicitly by the developer.

## Examples

### Importing a module in the crate root

Filename : `src/main.nr`

```rust
mod foo;

fn main() {
    foo::hello_world();
}
```

Filename : `src/foo.nr`

```rust
fn from_foo() {}
```

In the above snippet, the crate root is the `src/main.nr` file. The compiler sees the module
declaration `mod foo` which prompts it to look for a foo.nr file.

Visually this module hierarchy looks like the following :

```
crate
 ├── main
 │
 └── foo
      └── from_foo

```

The module filename may also be the name of the module as a directory with the contents in a 
file named `mod.nr` within that directory. The above example can alternatively be expressed like this:

Filename : `src/main.nr`

```rust
mod foo;

fn main() {
    foo::hello_world();
}
```

Filename : `src/foo/mod.nr`

```rust
fn from_foo() {}
```

Note that it's an error to have both files `src/foo.nr` and `src/foo/mod.nr` in the filesystem.

### Importing a module throughout the tree

All modules are accessible from the `crate::` namespace.

```
crate
 ├── bar
 ├── foo
 └── main

```

In the above snippet, if `bar` would like to use functions in `foo`, it can do so by `use crate::foo::function_name`.

### Sub-modules

Filename : `src/main.nr`

```rust
mod foo;

fn main() {
    foo::from_foo();
}
```

Filename : `src/foo.nr`

```rust
mod bar;
fn from_foo() {}
```

Filename : `src/foo/bar.nr`

```rust
fn from_bar() {}
```

In the above snippet, we have added an extra module to the module tree; `bar`. `bar` is a submodule
of `foo` hence we declare bar in `foo.nr` with `mod bar`. Since `foo` is not the crate root, the
compiler looks for the file associated with the `bar` module in `src/foo/bar.nr`

Visually the module hierarchy looks as follows:

```
crate
 ├── main
 │
 └── foo
      ├── from_foo
      └── bar
           └── from_bar
```

Similar to importing a module in the crate root, modules can be placed in a `mod.nr` file, like this:

Filename : `src/main.nr`

```rust
mod foo;

fn main() {
    foo::from_foo();
}
```

Filename : `src/foo/mod.nr`

```rust
mod bar;
fn from_foo() {}
```

Filename : `src/foo/bar/mod.nr`

```rust
fn from_bar() {}
```

### Referencing a parent module 

Given a submodule, you can refer to its parent module using the `super` keyword.

Filename : `src/main.nr`

```rust
mod foo;

fn main() {
    foo::from_foo();
}
```

Filename : `src/foo.nr`

```rust
mod bar;

fn from_foo() {}
```

Filename : `src/foo/bar.nr`

```rust
// Same as foo::from_foo
use super::from_foo; 

fn from_bar() {
    from_foo();        // invokes super::from_foo(), which is foo::from_foo()
    super::from_foo(); // also invokes foo::from_foo()
}
```

## Import Syntax

The `use` keyword brings items from other modules into scope. There are several forms:

### Basic imports

```rust
use crate::foo::bar;         // Import 'bar' from the 'foo' module in the crate root
use super::sibling_fn;       // Import from the parent module
use some_dependency::item;   // Import from a dependency (plain path)
```

### Aliases

You can rename an import with `as`:

```rust
use crate::foo::bar as my_bar;
```

You can also alias a trait to `_` to import its methods without bringing the trait name into scope:

```rust
use crate::foo::MyTrait as _;
```

This is useful when you need to call methods defined by a trait but don't want the trait name to be accessible in the current module.

### Grouped imports

Import multiple items from the same path:

```rust
use crate::foo::{bar, baz};
use crate::{foo::{bar2 as b, baz}, qux::{c, d}};
```

### Path prefixes

| Prefix | Meaning |
|--------|---------|
| `crate::` | From the root of the current crate |
| `super::` | From the parent module |
| `::` | An absolute path |
| _(plain)_ | Relative to the current module, or a dependency name |

### Re-exports

`use` declarations are private to the containing module by default. However, like functions,
they can be marked as `pub` or `pub(crate)`. A public `use` declaration serves to _re-export_ a name.
A public `use` declaration can therefore redirect some public name to a different target definition:
even a definition with a private canonical path, inside a different module.

An example of re-exporting:

```rust
mod some_module {
    pub use foo::{bar, baz};
    mod foo {
        pub fn bar() {}
        pub fn baz() {}
    }
}

fn main() {
    some_module::bar();
    some_module::baz();
}
```

In this example, the module `some_module` re-exports two public names defined in `foo`.

## The Prelude

Noir automatically imports a set of commonly used items into every file via the _prelude_. These items are available without an explicit `use` statement:

- `assert_constant`, `print`, `println` -- built-in functions
- `Eq`, `Ord` -- comparison traits (from `std::cmp`)
- `BoundedVec` -- bounded vector type (from `std::collections`)
- `From`, `Into` -- conversion traits (from `std::convert`)
- `Default` -- default value trait (from `std::default`)
- `derive`, `derive_via` -- derive macros (from `std::meta`)
- `Option` -- optional value type (from `std::option`)
- `panic` -- halt execution with a message (from `std::panic`)

### Visibility

By default, like functions, modules are private to the module (or crate) they exist in. You can use `pub`
to make the module public or `pub(crate)` to make it public to just its crate:

```rust
// This module is now public and can be seen by other crates.
pub mod foo;
```
