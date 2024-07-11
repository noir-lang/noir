---
title: Assert Function
description:
  Learn about the `assert` and `static_assert` functions in Noir, which can be used to explicitly
  constrain the predicate or comparison expression that follows to be true, and what happens if
  the expression is false at runtime or compile-time, respectively.
keywords: [Noir programming language, assert statement, predicate expression, comparison expression]
sidebar_position: 4
---

Noir includes a special `assert` function which will explicitly constrain the predicate/comparison
expression that follows to be true. If this expression is false at runtime, the program will fail to
be proven. Example:

```rust
fn main(x : Field, y : Field) {
    assert(x == y);
}
```

> Assertions only work for predicate operations, such as `==`. If there's any ambiguity on the operation, the program will fail to compile. For example, it is unclear if `assert(x + y)` would check for `x + y == 0` or simply would return `true`.

You can optionally provide a message to be logged when the assertion fails:

```rust
assert(x == y, "x and y are not equal");
```

Aside string literals, the optional message can be a format string or any other type supported as input for Noir's [print](../standard_library/logging.md) functions. This feature lets you incorporate runtime variables into your failed assertion logs:

```rust
assert(x == y, f"Expected x == y, but got {x} == {y}");
```

Using a variable as an assertion message directly:

```rust
struct myStruct {
  myField: Field
}

let s = myStruct { myField: y };
assert(s.myField == x, s);
```

There is also a special `static_assert` function that behaves like `assert`,
but that runs at compile-time.

```rust
fn main(xs: [Field; 3]) {
    let x = 2 + 2;
    let y = 4;
    static_assert(x == y, "expected 2 + 2 to equal 4");

    // This passes since the length of `xs` is known at compile-time
    static_assert(xs.len() == 3, "expected the input to have 3 elements");
}
```

This function fails when passed a dynamic (run-time) argument:

```rust
fn main(x : Field, y : Field) {
    // this fails because `x` is not known at compile-time
    static_assert(x == 2, "expected x to be known at compile-time and equal to 2");

    let mut example_slice = &[];
    if y == 4 {
        example_slice = example_slice.push_back(0);
    }

    // This fails because the length of `example_slice` is not known at
    // compile-time
    let error_message = "expected an empty slice, known at compile-time";
    static_assert(example_slice.len() == 0, error_message);
}
```

