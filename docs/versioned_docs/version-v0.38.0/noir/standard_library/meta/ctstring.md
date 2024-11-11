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

```rust title="as-ctstring" showLineNumbers 
pub trait AsCtString {
    comptime fn as_ctstring(self) -> CtString;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L43-L47" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L43-L47</a></sub></sup>


Converts an object into a compile-time string.

Implementations:

```rust
impl<let N: u32> AsCtString for str<N> { ... }
impl<let N: u32, T> AsCtString for fmtstr<N, T> { ... }
```

## Methods

### new

```rust title="new" showLineNumbers 
pub comptime fn new() -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L4-L6" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L4-L6</a></sub></sup>


Creates an empty `CtString`.

### append_str

```rust title="append_str" showLineNumbers 
pub comptime fn append_str<let N: u32>(self, s: str<N>) -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L11-L13</a></sub></sup>


Returns a new CtString with the given str appended onto the end.

### append_fmtstr

```rust title="append_fmtstr" showLineNumbers 
pub comptime fn append_fmtstr<let N: u32, T>(self, s: fmtstr<N, T>) -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L17-L19" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L17-L19</a></sub></sup>


Returns a new CtString with the given fmtstr appended onto the end.

### as_quoted_str

```rust title="as_quoted_str" showLineNumbers 
pub comptime fn as_quoted_str(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L26-L28</a></sub></sup>


Returns a quoted string literal from this string's contents.

There is no direct conversion from a `CtString` to a `str<N>` since
the size would not be known. To get around this, this function can
be used in combination with macro insertion (`!`) to insert this string
literal at this function's call site.

Example:

```rust title="as_quoted_str_example" showLineNumbers 
let my_ctstring = "foo bar".as_ctstring();
            let my_str = my_ctstring.as_quoted_str!();

            assert_eq(crate::meta::type_of(my_str), quote { str<7> }.as_type());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/ctstring.nr#L92-L97" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/ctstring.nr#L92-L97</a></sub></sup>


## Trait Implementations

```rust
impl Eq for CtString
impl Hash for CtString
impl Append for CtString
```
