---
title: Functions
description:
  Learn how to declare functions and methods in Noir, a programming language with Rust semantics.
  This guide covers parameter declaration, return types, call expressions, and more.
keywords: [Noir, Rust, functions, methods, parameter declaration, return types, call expressions]
sidebar_position: 1
---

Functions in Noir follow the same semantics of Rust, though Noir does not support early returns.

To declare a function the `fn` keyword is used.

```rust
fn foo() {}
```

By default, functions are visible only within the package they are defined. To make them visible outside of that package (for example, as part of a [library](../modules_packages_crates/crates_and_packages.md#libraries)), you should mark them as `pub`:

```rust
pub fn foo() {}
```

You can also restrict the visibility of the function to only the crate it was defined in, by specifying `pub(crate)`:

```rust
pub(crate) fn foo() {}  //foo can only be called within its crate
```

All parameters in a function must have a type and all types are known at compile time. The parameter
is prepended with a colon and the parameter type. Multiple parameters are separated using a comma.

```rust
fn foo(x : Field, y : Field){}
```

You can use an underscore `_` as a parameter name when you don't need to use the parameter in the function body. This is useful when you need to satisfy a function signature but don't need to use all the parameters:

```rust
fn foo(_ : Field, y : Field) {
    // Only using y parameter
}
```

Alternatively, you can prefix a parameter name with an underscore (e.g. `_x`), which also indicates that the parameter is unused. This approach is often preferred as it preserves the parameter name for documentation purposes:

```rust
fn foo(_x : Field, y : Field) -> Field {
    // Only using y parameter
    y
}
```

The return type of a function can be stated by using the `->` arrow notation. The function below
states that the foo function must return a `Field`. If the function returns no value, then the arrow
is omitted.

```rust
fn foo(x : Field, y : Field) -> Field {
    x + y
}
```

Note that a `return` keyword is unneeded in this case - the last expression in a function's body is
returned.

## Main function

If you're writing a binary, the `main` function is the starting point of your program. You can pass all types of expressions to it, as long as they have a fixed size at compile time:

```rust
fn main(x : Field) // this is fine: passing a Field
fn main(x : [Field; 2]) // this is also fine: passing a Field with known size at compile-time
fn main(x : (Field, bool)) // 👌: passing a (Field, bool) tuple means size 2
fn main(x : str<5>) // this is fine, as long as you pass a string of size 5

fn main(x : Vec<Field>) // can't compile, has variable size
fn main(x : [Field]) // can't compile, has variable size
fn main(....// i think you got it by now
```

Keep in mind [tests](../../tooling/tests.md) don't differentiate between `main` and any other function. The following snippet passes tests, but won't compile or prove:

```rust
fn main(x : [Field]) {
    assert(x[0] == 1);
}

#[test]
fn test_one() {
    main(&[1, 2]);
}
```

```bash
$ nargo test
[testing] Running 1 test functions
[testing] Testing test_one... ok
[testing] All tests passed

$ nargo check
The application panicked (crashed).
Message:  Cannot have variable sized arrays as a parameter to main
```

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
    assert(s.sum() == 42);
}
```

Methods are just syntactic sugar for functions, so if we wanted to we could also call `sum` as
follows:

```rust
assert(MyStruct::sum(s) == 42);
```

It is also possible to specialize which method is chosen depending on the [generic](./generics.md) type that is used. In this example, the `foo` function returns different values depending on its type:

```rust
struct Foo<T> {}

impl Foo<u32> {
    fn foo(self) -> Field { 1 }
}

impl Foo<u64> {
    fn foo(self) -> Field { 2 }
}

fn main() {
    let f1: Foo<u32> = Foo{};
    let f2: Foo<u64> = Foo{};
    assert(f1.foo() + f2.foo() == 3);
}
```

Also note that impls with the same method name defined in them cannot overlap. For example, if we already have `foo` defined for `Foo<u32>` and `Foo<u64>` like we do above, we cannot also define `foo` in an `impl<T> Foo<T>` since it would be ambiguous which version of `foo` to choose.

```rust
// Including this impl in the same project as the above snippet would
// cause an overlapping impls error
impl<T> Foo<T> {
    fn foo(self) -> Field { 3 }
}
```

## Lambdas

Lambdas are anonymous functions. They follow the syntax of Rust - `|arg1, arg2, ..., argN| return_expression`.

```rust
let add_50 = |val| val + 50;
assert(add_50(100) == 150);
```

See [Lambdas](./lambdas.md) for more details.

## Attributes

Attributes are metadata that can be applied to a function, using the following syntax: `#[attribute(value)]`.

See [Attributes](./attributes.md) for more details.
