---
title: Strings
description:
  Discover the String data type in Noir. Learn about its methods, see real-world examples, and understand how to effectively manipulate and use Strings in Noir.
keywords:
  [
    noir,
    string type,
    methods,
    examples,
    concatenation,
  ]
sidebar_position: 3
---


The string type is a fixed length value defined with `str<N>`.

You can use strings in `assert()` functions or print them with
`println()`. See more about [Logging](../../standard_library/logging.md).

```rust

fn main(message : pub str<11>, hex_as_string : str<4>) {
    println(message);
    assert(message == "hello world");
    assert(hex_as_string == "0x41");
}
```

You can convert a `str<N>` to a byte array by calling `as_bytes()`
or a vector by calling `as_bytes_vec()`.

```rust
fn main() {
    let message = "hello world";
    let message_bytes = message.as_bytes();
    let mut message_vec = message.as_bytes_vec();
    assert(message_bytes.len() == 11);
    assert(message_bytes[0] == 104);
    assert(message_bytes[0] == message_vec.get(0));
}
```

## Escape characters

You can use escape characters for your strings:

| Escape Sequence | Description     |
|-----------------|-----------------|
| `\r`            | Carriage Return |
| `\n`            | Newline         |
| `\t`            | Tab             |
| `\0`            | Null Character  |
| `\"`            | Double Quote    |
| `\\`            | Backslash       |

Example:

```rust
let s = "Hello \"world" // prints "Hello "world"
let s = "hey \tyou"; // prints "hey   you"
```

## Raw strings

A raw string begins with the letter `r` and is optionally delimited by a number of hashes `#`.

Escape characters are *not* processed within raw strings. All contents are interpreted literally.

Example:

```rust
let s = r"Hello world";
let s = r#"Simon says "hello world""#;

// Any number of hashes may be used (>= 1) as long as the string also terminates with the same number of hashes
let s = r#####"One "#, Two "##, Three "###, Four "####, Five will end the string."#####; 
```
