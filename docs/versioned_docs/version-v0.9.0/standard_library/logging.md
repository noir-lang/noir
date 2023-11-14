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

# Logging

The standard library provides a familiar `println` statement you can use. Despite being a limited
implementation of rust's `println!` macro, this construct can be useful for debugging.

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
