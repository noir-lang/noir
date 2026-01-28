---
title: Comments
description:
  Learn how to write comments in Noir programming language. A comment is a line of code that is
  ignored by the compiler, but it can be read by programmers. Single-line and multi-line comments
  are supported in Noir.
keywords: [Noir programming language, comments, single-line comments, multi-line comments]
sidebar_position: 10
---

A comment is a line in your codebase which the compiler ignores, however it can be read by
programmers.

Here is a single line comment:

```rust
// This is a comment and is ignored
```

`//` is used to tell the compiler to ignore the rest of the line.

Noir also supports multi-line block comments. Start a block comment with `/*` and end the block with `*/`.

Noir does not natively support doc comments. You may be able to use [Rust doc comments](https://doc.rust-lang.org/reference/comments.html) in your code to leverage some Rust documentation build tools with Noir code.

```rust
/*
  This is a block comment describing a complex function.
*/
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```
