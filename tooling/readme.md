# Structure

Below we briefly describe the purpose of each tool-related crate in this repository.

## nargo

This is the default package manager used by Noir. One may draw similarities to Rust's Cargo.

## nargo_fmt

This is the default formatter used by Noir, analogous to Rust's rustfmt.

## lsp

This is the platform agnostic implementation of Noir's Language Server. It implements the various features supported, but doesn't bind to any particular transport. Binding to a transport must be done when consuming the crate.
