---
title: fmtstr
---

`fmtstr<N, T>` is the type resulting from using format string (`f"..."`).

## Methods

### quoted_contents

```rust title="quoted_contents" showLineNumbers 
pub comptime fn quoted_contents(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/format_string.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/format_string.nr#L3-L5</a></sub></sup>


Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.