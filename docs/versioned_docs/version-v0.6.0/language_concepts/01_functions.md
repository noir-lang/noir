---
title: Functions
description:
  Learn how to declare functions and methods in Noir, a programming language with Rust semantics.
  This guide covers parameter declaration, return types, call expressions, and more.
keywords: [Noir, Rust, functions, methods, parameter declaration, return types, call expressions]
---

Functions in Noir follow the same semantics of Rust, though Noir does not support early returns.

To declare a function the `fn` keyword is used.

```rust
fn foo() {}
```

All parameters in a function must have a type and all types are known at compile time. The parameter
is pre-pended with a colon and the parameter type. Multiple parameters are separated using a comma.

```rust
fn foo(x : Field, y : pub Field){}
```

The return type of a function can be stated by using the `->` arrow notation. The function below
states that the foo function must return a `Field`. If the function returns no value, then the arrow
is omitted.

```rust
fn foo(x : Field, y : pub Field) -> Field {
    x + y
}
```

Note that a `return` keyword is unneeded in this case - the last expression in a function's body is
returned.

## Call Expressions

Calling a function in Noir is executed by using the function name and passing in the necessary
arguments.

Below we show how to call the `foo` function from the `main` function using a call expression:

```rust
fn main(x : Field, y : Field) {
    let z = foo(x);
}

fn foo(x : Field) -> Field {
    x + x
}
```

## Methods

You can define methods in Noir on any struct type in scope.

```rust
struct MyStruct {
    foo: Field,
    bar: Field,
}

impl MyStruct {
    fn new(foo: Field) -> MyStruct {
        MyStruct {
            foo,
            bar: 2,
        }
    }

    fn sum(self) -> Field {
        self.foo + self.bar
    }
}

fn main() {
    let s = MyStruct::new(40);
    constrain s.sum() == 42;
}
```

Methods are just syntactic sugar for functions, so if we wanted to we could also call `sum` as
follows:

```rust
constrain MyStruct::sum(s) == 42
```
