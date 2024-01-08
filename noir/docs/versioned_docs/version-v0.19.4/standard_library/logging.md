---
title: Logging
description:
  Learn how to use the println statement for debugging in Noir with this tutorial. Understand the
  basics of logging in Noir and how to implement it in your code.
keywords:
  [
    noir logging,
    println statement,
    debugging in noir,
    noir std library,
    logging tutorial,
    basic logging in noir,
    noir logging implementation,
    noir debugging techniques,
    rust,
  ]
---

The standard library provides a familiar `println` statement you can use. Despite being a limited
implementation of rust's `println!` macro, this construct can be useful for debugging.

You can print the output of println statements in your Noir code by using the `nargo execute` command or the `--show-output` flag when using `nargo test` (provided there are println statements in your tests).

It is recommended to use `nargo execute` if you want to debug failing constrains with `println` statements. This is due to every input in a test being a constant rather than a witness, so we issue an error during compilation while we only print during execution (which comes after compilation). `println` will not work for failed constraints caught at compile time.

The `println` statement is unconstrained, so it works for outputting integers, fields, strings, and even structs or expressions. For example:

```rust
use dep::std;

struct Person {
    age : Field,
    height : Field,
}

fn main(age : Field, height : Field) {
    let person = Person { age : age, height : height };
    std::println(person);
    std::println(age + height);
    std::println("Hello world!");
}

```

You can print multiple different types in the same statement and string as well as a new "fmtstr" type. A `fmtstr` can be specified in the same way as a normal string it just should be prepended with an "f" character:

```rust
  let fmt_str = f"i: {i}, j: {j}";
  std::println(fmt_str);

  let s = myStruct { y: x, x: y };
  std::println(s);

  std::println(f"i: {i}, s: {s}");

  std::println(x);
  std::println([x, y]);

  let foo = fooStruct { my_struct: s, foo: 15 };
  std::println(f"s: {s}, foo: {foo}");
```
