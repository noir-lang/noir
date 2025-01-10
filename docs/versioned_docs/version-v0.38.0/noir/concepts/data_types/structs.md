---
title: Structs
description:
  Explore the Struct data type in Noir. Learn about its methods, see real-world examples, and grasp how to effectively define and use Structs in your Noir programs.
keywords:
  [
    noir,
    struct type,
    methods,
    examples,
    data structures,
  ]
sidebar_position: 8
---

A struct also allows for grouping multiple values of different types. Unlike tuples, we can also
name each field.

> **Note:** The usage of _field_ here refers to each element of the struct and is unrelated to the
> field type of Noir.

Defining a struct requires giving it a name and listing each field within as `<Key>: <Type>` pairs:

```rust
struct Animal {
    hands: Field,
    legs: Field,
    eyes: u8,
}
```

An instance of a struct can then be created with actual values in `<Key>: <Value>` pairs in any
order. Struct fields are accessible using their given names:

```rust
fn main() {
    let legs = 4;

    let dog = Animal {
        eyes: 2,
        hands: 0,
        legs,
    };

    let zero = dog.hands;
}
```

Structs can also be destructured in a pattern, binding each field to a new variable:

```rust
fn main() {
    let Animal { hands, legs: feet, eyes } = get_octopus();

    let ten = hands + feet + eyes as u8;
}

fn get_octopus() -> Animal {
    let octopus = Animal {
        hands: 0,
        legs: 8,
        eyes: 2,
    };

    octopus
}
```

The new variables can be bound with names different from the original struct field names, as
showcased in the `legs --> feet` binding in the example above.

### Visibility

By default, like functions, structs are private to the module they exist in. You can use `pub`
to make the struct public or `pub(crate)` to make it public to just its crate:

```rust
// This struct is now public
pub struct Animal {
    hands: Field,
    legs: Field,
    eyes: u8,
}
```

The same applies to struct fields: by default they are private to the module they exist in,
but they can be made `pub` or `pub(crate)`:

```rust
// This struct is now public
pub struct Animal {
    hands: Field,           // private to its module
    pub(crate) legs: Field, // accessible from the entire crate
    pub eyes: u8,           // accessible from anywhere
}
```