---
title: Data Bus
sidebar_position: 12
---
**Disclaimer** this feature is experimental, do not use it!

The data bus is an optimization that the backend can use to make recursion more efficient.
In order to use it, you must define some inputs of the program entry points (usually the `main()`
function) with the `call_data` modifier, and the return values with the `return_data` modifier.
These modifiers are incompatible with `pub` and `mut` modifiers.

## Example

```rust
fn main(mut x: u32, y: call_data u32, z: call_data [u32;4] ) -> return_data u32 {
  let a = z[x];
  a+y
}
```

As a result, both call_data and return_data will be treated as private inputs and encapsulated into a read-only array each, for the backend to process.
