---
title: Distinct Witnesses
---

The `distinct` keyword prevents repetitions of witness indices in the program's ABI. This ensures
that the witnesses being returned as public inputs are all unique.

The `distinct` keyword is only used for return values on program entry points (usually the `main()`
function).

When using `distinct` and `pub` simultaneously, `distinct` comes first. See the example below.

You can read more about the problem this solves
[here](https://github.com/noir-lang/noir/issues/1183).

## Example

Without the `distinct` keyword, the following program

```rust
fn main(x : pub Field, y : pub Field) -> pub [Field; 4] {
    let a = 1;
    let b = 1;
    [x + 1, y, a, b]
}
```

compiles to

```json
{
  //...
  "abi": {
    //...
    "param_witnesses": { "x": [1], "y": [2] },
    "return_witnesses": [3, 2, 4, 4]
  }
}
```

Whereas (with the `distinct` keyword)

```rust
fn main(x : pub Field, y : pub Field) -> distinct pub [Field; 4] {
    let a = 1;
    let b = 1;
    [x + 1, y, a, b]
}
```

compiles to

```json
{
  //...
  "abi": {
    //...
    "param_witnesses": { "x": [1], "y": [2] },
    //...
    "return_witnesses": [3, 4, 5, 6]
  }
}
```
