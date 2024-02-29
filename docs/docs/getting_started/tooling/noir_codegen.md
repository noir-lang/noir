---
title: Noir Codegen for Typescript
description: Learn how to use Noir codegen to generate Typescript interfaces 
keywords: [Nargo, Noir, compile, TypeScript]
sidebar_position: 2
---

You can generate TypeScript bindings with `noir_codegen`, allowing for easier and faster integration of your Noir circuits into TypeScript projects.

It is used in conjuction with `nargo export` which generates JSON for specified functions.

## Installation

You can install `noir_codegen` into your project using `yarn` or `npm`:

```bash
yarn add @noir-lang/noir_codegen
```

or 

```bash
npm install @noir-lang/noir_codegen
```
## Usage

Before generating TypeScript bindings, you need to export your Noir programs using `nargo export`:

```bash
nargo export
```

You can run this in the directory where your `Nargo.toml` lives, or you can specify a path like so:

```bash
nargo export --program-dir=./path/to/your/noir/program
```

Now you can run `noir-codegen`:

<!-- TODO: currently cannot get this working -->

## Example

For example, if you have a Noir program like this:

```rust
#[export]
fn exported_function_foo(x: u64, y: u64, array: [u8; 5], my_struct: NestedStruct, string: str<5>) -> (u64, u64, MyStruct) {
    assert(array.len() == 5);
    assert(my_struct.foo.foo);
    assert(string == "12345");

    print(x);
    assert(x < y);
    (x + y, 3, my_struct.foo)
}

fn unexported_function(x: u64, y: u64) {
    assert(x != y);
}
```

You can run `nargo export` in the directory with your `Nargo.toml` and


### Example Usage

Here's an example command that demonstrates how to use `nargo export` with these arguments:

```sh
nargo export --out-dir ./path/to/output '**/*.json'
