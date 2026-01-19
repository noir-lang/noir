---
title: Booleans
description:
  Delve into the Boolean data type in Noir. Understand its methods, practical examples, and best practices for using Booleans in your Noir programs.
keywords:
  [
    noir,
    boolean type,
    methods,
    examples,
    logical operations,
  ]
sidebar_position: 2
---


The `bool` type in Noir has two possible values: `true` and `false`:

```rust
fn main() {
    let t = true;
    let f: bool = false;
}
```

The boolean type is most commonly used in conditionals like `if` expressions and `assert`
statements. More about conditionals is covered in the [Control Flow](../control_flow.md) and
[Assert Function](../assert.md) sections.
