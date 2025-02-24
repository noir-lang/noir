---
title: fmtstr
---

`fmtstr<N, T>` is the type resulting from using format string (`f"..."`).

## Methods

### quoted_contents

#include_code quoted_contents noir_stdlib/src/meta/format_string.nr rust

Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.