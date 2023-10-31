---
title: Testing in Noir
description: Learn how to use Nargo to test your Noir program in a quick and easy way
keywords: [Nargo, testing, Noir, compile, test]
---

You can test your Noir programs using Noir circuits.

Nargo will automatically compile and run any functions which have the decorator `#[test]` on them if
you run `nargo test`.

For example if you have a program like:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test]
fn test_add() {
    assert(add(2,2) == 4);
    assert(add(0,1) == 1);
    assert(add(1,0) == 1);
}
```

Running `nargo test` will test that the `test_add` function can be executed while satisfying the all
the contraints which allows you to test that add returns the expected values. Test functions can't
have any arguments currently.

### Test fail

You can write tests that are expected to fail by using the decorator `#[test(should_fail)]`. For example:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test(should_fail)]
fn test_add() {
    assert(add(2,2) == 5);
}
```
