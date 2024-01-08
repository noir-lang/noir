---
title: Modules
description:
  Learn how to organize your files using modules in Noir, following the same convention as Rust's
  module system. Examples included.
keywords: [Noir, Rust, modules, organizing files, sub-modules]
---

Noir's module system follows the same convention as the _newer_ version of Rust's module system.

## Purpose of Modules

Modules are used to organise files. Without modules all of your code would need to live in a single
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
