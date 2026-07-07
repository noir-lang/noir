---
title: fmtstr
---

`fmtstr<N, T>` is the type resulting from using format string (`f"..."`).

## Methods

### quoted_contents

```rust title="quoted_contents" showLineNumbers 
pub comptime fn quoted_contents(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/format_string.nr#L6-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/format_string.nr#L6-L8</a></sub></sup>


Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.

### as_quoted_str

```rust title="as_quoted_str" showLineNumbers 
pub comptime fn as_quoted_str(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/format_string.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/format_string.nr#L11-L13</a></sub></sup>


Returns the format string contents (with the leading and trailing double quotes) as a `Quoted` string literal (not a format string literal).

Example:

```rust title="as_quoted_str_test" showLineNumbers 
comptime {
        let x = 1;
        let f: str<_> = f"x = {x}".as_quoted_str!();
        assert_eq(f, "x = 0x01");
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_fmt_strings/src/main.nr#L19-L25" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_fmt_strings/src/main.nr#L19-L25</a></sub></sup>

