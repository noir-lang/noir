---
title: Control Flow
description:
  Learn how to use loops and if expressions in the Noir programming language. Discover the syntax
  and examples for for loops and if-else statements.
keywords: [Noir programming language, loops, for loop, if-else statements, Rust syntax]
sidebar_position: 2
---

## If Expressions

Noir supports `if-else` statements. The syntax is most similar to Rust's where it is not required
for the statement's conditional to be surrounded by parentheses.

```rust
let a = 0;
let mut x: u32 = 0;

if a == 0 {
    if a != 0 {
        x = 6;
    } else {
        x = 2;
    }
} else {
    x = 5;
    assert(x == 5);
}
assert(x == 2);
```

## For loops

`for` loops allow you to repeat a block of code multiple times.

The following block of code between the braces is run 10 times.

```rust
for i in 0..10 {
    // do something
}
```

Alternatively, `start..=end` can be used for a range that is inclusive on both ends.

The index for loops is of type `u64`.

### Break and Continue

In unconstrained code, `break` and `continue` are also allowed in `for` and `loop` loops. These are only allowed
in unconstrained code since normal constrained code requires that Noir knows exactly how many iterations
a loop may have. `break` and `continue` can be used like so:

```rust
for i in 0 .. 10 {
    println("Iteration start")

    if i == 2 {
        continue;
    }

    if i == 5 {
        break;
    }

    println(i);
}
println("Loop end")
```

When used, `break` will end the current loop early and jump to the statement after the for loop. In the example
above, the `break` will stop the loop and jump to the `println("Loop end")`.

`continue` will stop the current iteration of the loop, and jump to the start of the next iteration. In the example
above, `continue` will jump to `println("Iteration start")` when used. Note that the loop continues as normal after this.
The iteration variable `i` is still increased by one as normal when `continue` is used.

`break` and `continue` cannot currently be used to jump out of more than a single loop at a time.

## Loops

In unconstrained code, `loop` is allowed for loops that end with a `break`. This is only allowed
in unconstrained code since normal constrained code requires that Noir knows exactly how many iterations
a loop may have.

```rust
let mut i = 10
loop {
    println(i);
    i -= 1;

    if i == 0 {
        break;
    }
}
```

