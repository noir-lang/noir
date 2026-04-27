---
title: Standard Library
description: Overview of Noir standard library documentation and where to find common modules, types, and functions.
sidebar_position: 0
---

Use this section when you need built-in library support rather than language syntax.

## Common APIs

- [Traits](./traits.md) - common standard library traits such as `Default`, `From`, `Into`, `Eq`, `Ord`, and operator traits.
- [Option<T> Type](./options.md) - optional values.
- [Logging and Panics](./logging.md) - `print`, `println`, and panic behavior.
- [Memory Module](./mem.md) - low-level memory helpers.
- [Is Unconstrained Function](./is_unconstrained.md) - detect unconstrained execution.

## Collections and Cryptography

- [Containers](./containers/index.md) - bounded vectors and hash maps.
- [Cryptographic Primitives](./cryptographic_primitives/index.md) - ciphers, hashes, signature verification, and embedded curve operations.
- [Black Box Functions](./black_box_fns.md) - backend-supported specialized constraints.
- [Recursive Proofs](./recursion.mdx) - recursive verification helpers.

## Metaprogramming

- [Metaprogramming](./meta/index.md) - `std::meta` types and functions used with compile-time code.
