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
---


The string type is a fixed length value defined with `str<N>`.

You can use strings in `assert()` functions or print them with
`std::println()`. See more about [Logging](../../standard_library/logging).

```rust
use dep::std;

fn main(message : pub str<11>, hex_as_string : str<4>) {
    std::println(message);
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
