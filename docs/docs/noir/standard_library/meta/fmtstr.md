---
title: fmtstr
---

`std::meta::format_string` contains comptime methods on the `fmtstr` type for format strings.

## Methods

### contents

#include_code quoted noir_stdlib/src/meta/format_string.nr rust

Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.