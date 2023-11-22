---
title: Crates and Packages
description:
  Learn how to use Crates and Packages in your Noir project
keywords: [Nargo, dependencies, package management, crates, package]
---

## Crates

A crate is the smallest amount of code that the Noir compiler considers at a time.
Crates can contain modules, and the modules may be defined in other files that get compiled with the crate, as we’ll see in the coming sections.

### Crate Types

A Noir crate can come in one of two forms: a binary crate or a library crate.

_Binary crates_ are programs which you can compile to an ACIR circuit which you can then create proofs against. Each must have a function called `main` that defines the ACIR circuit which is to be proved.

_Library crates_ don't have a `main` function and they don't compile down to ACIR. Instead they define functionality intended to be shared with multiple projects, and eventually included in a binary crate.

### Crate Root

Every crate has a root, which is the source file that the compiler starts, this is also known as the root module. The Noir compiler does not enforce any conditions on the name of the file which is the crate root, however if you are compiling via Nargo the crate root must be called `lib.nr` or `main.nr` for library or binary crates respectively.

## Packages

A Nargo _package_ is a collection of one of more crates that provides a set of functionality. A package must include a Nargo.toml file.

A package _must_ contain either a library or a binary crate, but not both.

### Differences from Cargo Packages

One notable difference between Rust's Cargo and Noir's Nargo is that while Cargo allows a package to contain an unlimited number of binary crates and a single library crate, Nargo currently only allows a package to contain a single crate.

In future this restriction may be lifted to allow a Nargo package to contain both a binary and library crate or multiple binary crates.
