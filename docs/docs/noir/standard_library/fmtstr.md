---
title: fmtstr
description: Format string literals at compile time—inspect raw contents or emit quoted strings for macro generation.
---

`fmtstr<N, T>` is the type resulting from using format string (`f"..."`).

## Methods

### quoted_contents

#include_code quoted_contents noir_stdlib/src/meta/format_string.nr rust

Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.

### as_quoted_str

#include_code as_quoted_str noir_stdlib/src/meta/format_string.nr rust

Returns the format string contents (with the leading and trailing double quotes) as a `Quoted` string literal (not a format string literal).

Example:

#include_code as_quoted_str_test test_programs/compile_success_empty/comptime_fmt_strings/src/main.nr rust
