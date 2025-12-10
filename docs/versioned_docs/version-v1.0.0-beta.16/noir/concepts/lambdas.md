---
title: Lambdas
description: Learn how to use anonymous functions in Noir programming language.
keywords: [Noir programming language, lambda, closure, function, anonymous function]
sidebar_position: 9
---

## Introduction

Lambdas are anonymous functions. The syntax is `|arg1, arg2, ..., argN| return_expression`.

```rust
let add_50 = |val| val + 50;
assert(add_50(100) == 150);
```

A block can be used as the body of a lambda, allowing you to declare local variables inside it:

```rust
let cool = || {
  let x = 100;
  let y = 100;
  x + y
}

assert(cool() == 200);
```

## Closures

Inside the body of a lambda, you can use variables defined in the enclosing function. Such lambdas are called **closures**. In this example `x` is defined inside `main` and is accessed from within the lambda:

```rust
fn main() {
  let x = 100;
  let closure = || x + 150;
  assert(closure() == 250);
}
```

## Passing closures to higher-order functions

It may catch you by surprise that the following code fails to compile:

```rust
fn foo(f: fn () -> Field) -> Field {
 f()
}

fn main() {
  let (x, y) = (50, 50);
  assert(foo(|| x + y) == 100); // error :(
}
```

The reason is that the closure's capture environment affects its type - we have a closure that captures two Fields and `foo`
expects a regular function as an argument - those are incompatible.
:::note

Variables contained within the `||` are the closure's parameters, and the expression that follows it is the closure's body. The capture environment is comprised of any variables used in the closure's body that are not parameters.

E.g. in |x| x + y, y would be a captured variable, but x would not be, since it is a parameter of the closure.

:::
The syntax for the type of a closure is `fn[env](args) -> ret_type`, where `env` is the capture environment of the closure -
in this example that's `(Field, Field)`.

The best solution in our case is to make `foo` generic over the environment type of its parameter, so that it can be called
with closures with any environment, as well as with regular functions:

```rust
fn foo<Env>(f: fn[Env]() -> Field) -> Field {
 f()
}

fn main() {
  let (x, y) = (50, 50);
  assert(foo(|| x + y) == 100); // compiles fine
  assert(foo(|| 60) == 60);     // compiles fine
}
```
