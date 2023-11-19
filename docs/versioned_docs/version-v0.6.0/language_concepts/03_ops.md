---
title: Logical Operations
description:
  Learn about the supported arithmetic and logical operations in the Noir programming language.
  Discover how to perform operations on private input types, integers, and booleans.
keywords:
  [
    Noir programming language,
    supported operations,
    arithmetic operations,
    logical operations,
    predicate operators,
    bitwise operations,
    short-circuiting,
    backend,
  ]
---

# Operations

## Table of Supported Operations

| Operation |                          Description                           |                           Requirements |
| :-------- | :------------------------------------------------------------: | -------------------------------------: |
| +         |             Adds two private input types together              |            Types must be private input |
| -         |           Subtracts two private input types together           |            Types must be private input |
| \*        |          Multiplies two private input types together           |            Types must be private input |
| /         |            Divides two private input types together            |            Types must be private input |
| ^         |              XOR two private input types together              |                  Types must be integer |
| &         |              AND two private input types together              |                  Types must be integer |
| <<        |        Left shift an integer by another integer amount         |                  Types must be integer |
| >>        |        Right shift an integer by another integer amount        |                  Types must be integer |
| !         |                     Bitwise not of a value                     |        Type must be integer or boolean |
| <         |       returns a bool if one value is less than the other       | Upper bound must have a known bit size |
| <=        | returns a bool if one value is less than or equal to the other | Upper bound must have a known bit size |
| >         |       returns a bool if one value is more than the other       | Upper bound must have a known bit size |
| >=        | returns a bool if one value is more than or equal to the other | Upper bound must have a known bit size |
| ==        |       returns a bool if one value is equal to the other        |       Both types must not be constants |
| !=        |     returns a bool if one value is not equal to the other      |       Both types must not be constants |

### Predicate Operators

`<,<=, !=, == , >, >=` are known as predicate/comparison operations because they compare two values.
This differs from the operations such as `+` where the operands are used in _computation_.

### Bitwise Operations Example

```rust
fn main(x : Field) {
    let y = x as u32;
    let z = y & y;
}
```

`z` is implicitly constrained to be the result of `y & y`. The `&` operand is used to denote bitwise
`&`.

> `x & x` would not compile as `x` is a `Field` and not an integer type.

### Logical Operators

Noir has no support for the logical operators `||` and `&&`. This is because encoding the
short-circuiting that these operators require can be inefficient for Noir's backend. Instead you can
use the bitwise operators `|` and `&` which operate indentically for booleans, just without the
short-circuiting.

```rust
let my_val = 5;

let mut flag = 1;
if (my_val > 6) | (my_val == 0) {
    flag = 0;
}
constrain flag == 1;

if (my_val != 10) & (my_val < 50) {
    flag = 0;
}
constrain flag == 0;
```

### Shorthand operators

Noir shorthand operators for most of the above operators, namely `+=, -=, *=, /=, %=, &=, |=, ^=, <<=`, and `>>=`. These allow for more concise syntax. For example:

```rust
let mut i = 0;
i = i + 1;
```

could be written as:

```rust
let mut i = 0;
i += 1;
```
