---
title: Is Unconstrained Function
description:
  The is_unconstrained function returns wether the context at that point of the program is unconstrained or not.
keywords:
  [
    unconstrained
  ]
---

It's very common for functions in circuits to take unconstrained hints of an expensive computation and then verify it. This is done by running the hint in an unconstrained context and then verifying the result in a constrained context.

When a function is marked as unconstrained, any subsequent functions that it calls will also be run in an unconstrained context. However, if we are implementing a library function, other users might call it within an unconstrained context or a constrained one. Generally, in an unconstrained context we prefer just computing the result instead of taking a hint of it and verifying it, since that'd mean doing the same computation twice:

```rust 

fn my_expensive_computation(){
  ...
}

unconstrained fn my_expensive_computation_hint(){
  my_expensive_computation()
}

pub fn external_interface(){
  my_expensive_computation_hint();
  // verify my_expensive_computation: If external_interface is called from unconstrained, this is redundant
  ...
}

```

In order to improve the performance in an unconstrained context you can use the function at `std::runtime::is_unconstrained() -> bool`:


```rust 
use dep::std::runtime::is_unconstrained;

fn my_expensive_computation(){
  ...
}

unconstrained fn my_expensive_computation_hint(){
  my_expensive_computation()
}

pub fn external_interface(){
  if is_unconstrained() {
    my_expensive_computation();
  } else {
    my_expensive_computation_hint();
    // verify my_expensive_computation
    ...
  }
}

```

The is_unconstrained result is resolved at compile time, so in unconstrained contexts the compiler removes the else branch, and in constrained contexts the compiler removes the if branch, reducing the amount of compute necessary to run external_interface.

Note that using `is_unconstrained` in a `comptime` context will also return `true`:

```
fn main() {
    comptime {
        assert(is_unconstrained());
    }
}
```
