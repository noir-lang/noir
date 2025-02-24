---
title: CtString
---

`std::meta::ctstring` contains methods on the built-in `CtString` type which is
a compile-time, dynamically-sized string type. Compared to `str<N>` and `fmtstr<N, T>`,
`CtString` is useful because its size does not need to be specified in its type. This
can be used for formatting items at compile-time or general string handling in `comptime`
code.

Since `fmtstr`s can be converted into `CtString`s, you can make use of their formatting
abilities in CtStrings by formatting in `fmtstr`s then converting the result to a CtString
afterward.

## Traits

### AsCtString

#include_code as-ctstring noir_stdlib/src/meta/ctstring.nr rust

Converts an object into a compile-time string.

Implementations:

```rust
impl<let N: u32> AsCtString for str<N> { ... }
impl<let N: u32, T> AsCtString for fmtstr<N, T> { ... }
```

## Methods

### new

#include_code new noir_stdlib/src/meta/ctstring.nr rust

Creates an empty `CtString`.

### append_str

#include_code append_str noir_stdlib/src/meta/ctstring.nr rust

Returns a new CtString with the given str appended onto the end.

### append_fmtstr

#include_code append_fmtstr noir_stdlib/src/meta/ctstring.nr rust

Returns a new CtString with the given fmtstr appended onto the end.

### as_quoted_str

#include_code as_quoted_str noir_stdlib/src/meta/ctstring.nr rust

Returns a quoted string literal from this string's contents.

There is no direct conversion from a `CtString` to a `str<N>` since
the size would not be known. To get around this, this function can
be used in combination with macro insertion (`!`) to insert this string
literal at this function's call site.

Example:

#include_code as_quoted_str_example noir_stdlib/src/meta/ctstring.nr rust

## Trait Implementations

```rust
impl Eq for CtString
impl Hash for CtString
impl Append for CtString
```
