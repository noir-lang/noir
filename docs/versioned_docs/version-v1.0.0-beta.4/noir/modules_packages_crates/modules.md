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
// Same as bar::from_foo
use super::from_foo; 

fn from_bar() {
    from_foo();        // invokes super::from_foo(), which is bar::from_foo()
    super::from_foo(); // also invokes bar::from_foo()
}
```

### `use` visibility

`use` declarations are private to the containing module, by default. However, like functions, 
they can be marked as `pub` or `pub(crate)`. Such a use declaration serves to _re-export_ a name. 
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

### Visibility

By default, like functions, modules are private to the module (or crate) they exist in. You can use `pub`
to make the module public or `pub(crate)` to make it public to just its crate:

```rust
// This module is now public and can be seen by other crates.
pub mod foo;
```