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

By default, functions are visible only within the package they are defined. To make them visible outside of that package (for example, as part of a [library](../modules_packages_crates/crates_and_packages.md#libraries)), you should mark them as `pub`:

```rust
pub fn foo() {}
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

Keep in mind [tests](../nargo/02_testing.md) don't differentiate between `main` and any other function. The following snippet passes tests, but won't compile or prove:

```rust
fn main(x : [Field]) {
    assert(x[0] == 1);
}

#[test]
fn test_one() {
    main([1, 2]);
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

## Lambdas

Lambdas are anonymous functions. They follow the syntax of Rust - `|arg1, arg2, ..., argN| return_expression`.

```rust
let add_50 = |val| val + 50;
assert(add_50(100) == 150);
```

See [Lambdas](./08_lambdas.md) for more details.

## Attributes

Attributes are metadata that can be applied to a function, using the following syntax: `#[attribute(value)]`.

Supported attributes include:
- **builtin**: the function is implemented by the compiler, for efficiency purposes.
- **deprecated**: mark the function as *deprecated*. Calling the function will generate a warning: `warning: use of deprecated function`
- **field**: Used to enable conditional compilation of code depending on the field size. See below for more details
- **oracle**: mark the function as *oracle*; meaning it is an external unconstrained function, implemented in noir_js. See [Unconstrained](./05_unconstrained.md) and [Noir js](../noir_js/noir_js.md) for more details.
- **test**: mark the function as unit tests. See [Tests](../nargo/02_testing.md) for more details

### Field Attribute
The field attribute defines which field the function is compatible for. The function is conditionally compiled, under the condition that the field attribute matches the Noir native field.
The field can be defined implicitly, by using the name of the elliptic curve usually associated to it - for instance bn254, bls12_381 - or explicitly by using the field (prime) order, in decimal or hexadecimal form.
As a result, it is possible to define multiple versions of a function with each version specialized for a different field attribute. This can be useful when a function requires different parameters depending on the underlying elliptic curve.


Example: we define the function `foo()` three times below. Once for the default Noir bn254 curve, once for the field $\mathbb F_{23}$, which will normally never be used by Noir, and once again for the bls12_381 curve.
```rust
#[field(bn254)]
fn foo() -> u32 {
    1
}

#[field(23)]
fn foo() -> u32 {
    2
}

// This commented code would not compile as foo would be defined twice because it is the same field as bn254
// #[field(21888242871839275222246405745257275088548364400416034343698204186575808495617)]
// fn foo() -> u32 {
//     2
// }

#[field(bls12_381)]
fn foo() -> u32 {
    3
}
```

If the field name is not known to Noir, it will discard the function. Field names are case insensitive.