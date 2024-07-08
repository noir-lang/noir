---
title: Writing Noir
description: Understand new considerations when writing Noir
keywords: [Noir, programming, rust]
tags: [Optimisation]
sidebar_position: 3
---

# Writing Noir for fun and profit

This article intends to set you up with a key concept essential for writing more viable applications that use zero knowledge proofs, namely around non-wasteful circuits.

## Context - 'Efficient' is subjective

When writing a web application for a performant computer with high-speed internet connection, writing efficient code sometimes is seen as an afterthought only if needed. Large multiplications running at the innermost of nested loops may not even be on a dev's radar.
When writing firmware for a battery-powered microcontroller, you think of cpu cycles as rations to keep within a product's power budget.

> Code is written to create applications that perform specific tasks within specific constraints

And these constraints differ depending on where the compiled code is execute.
### The Ethereum Virtual Machine (EVM)

For the EVM, Solidity code is compiled into bytecode to be executed, and the gas cost of opcodes become as important as clock cycles. These gas costs were designed in to the protocol to reward machines that are executing bytecode for the EVM, so it is no surprise that a lot of the costs are roughly proportional to clock-cycles (a proxy for power consumption). Eg Addition: 3, Multiplication: 8.

But there is a twist, the cost of writing to disk is amplified, and is a few orders of magnitude larger. Namely: 20k for allocating and writing a new value, or 5k for writing an existing value.
So whilst writing efficient code for regular computers and the EVM is mostly the same, there are some key differences like this more immediate compensation of disk writes.

In scenarios where extremely low gas costs are required for an application to be viable/competitive, developers get into what is colloquially known as: "*gas golfing*". Finding the lowest execution cost to achieve a specific task.

### Coding for circuits - a paradigm shift

In zero knowledge cryptography, code is compiled to arithmetic gates called "circuits", and gate count is the significant cost. Depending on the backend this is linearly proportionate to proving time, and so from a product point this should be kept as low as possible.

Whilst writing efficient code for web apps and solidity has a few key differences, writing efficient circuits have a different set of considerations. It is a bit of a paradigm shift, like writing code for GPUs for the first time...

Eg drawing a circle at (0, 0) of radius `r`:
- For a single CPU thread,
```
for theta in 0..2*pi {
  let x = r * cos(theta);
  let y = r * sin(theta);
  draw(x, y);
} // note: would do 0 - pi/2 and draw +ve/-ve x and y.
```

- For GPUs (simultaneous parallel calls with x, y across image),
```
if (x^2 + y^2 = r^2) {
  draw(x, y);
}
```

([Related](https://www.youtube.com/watch?v=-P28LKWTzrI))

Whilst this CPU -> GPU does not translate to circuits exactly, it is intended to exemplify the difference in intuition when coding for different machine capabilities/constraints.

### Context Takeaway

For those coming from a primarily web app background, this article will explain what you need to consider when writing circuits. Furthermore, for those experienced writing efficient machine code, prepare to shift what you think is efficient 😬

## Code re-use

For some applications using Noir, existing code might be a convenient starting point to then proceed to optimise the gate count of.

:::note
Many valuable functions and algorithms have been written in more established languages (C/C++), and converted to modern ones (like Rust).
:::

Fortunately for Noir devs, when needing a particular function a rust implementation can be readily compiled into Noir with some key changes. While the compiler does a decent amount of optimisations, it won't be able to change code that has been optimized for clock-cycles into code optimized for arithmetic gates.

## Writing efficient Noir for performant products

The following points help refine our understanding over time.

:::note
> A Noir program makes a statement that can be verified.
:::

It compiles to a structure that represents the calculation, and can assert results within the calculation at any stage (via the `constrain` keyword).

A Noir program compiles to an Abstract Circuit Intermediate Representation which is:
 - A tree structure
 - Leaves (inputs) are the `Field` type
 - Nodes contain arithmetic operations to combine them (gates)
 - The root is the final result (return value)

:::tip
The command `nargo info` shows the programs circuit size, and is useful to compare the value of changes made.
Advanced: You can dig deeper and use the `--print-acir` param to take a closer look at individual gates too.
:::

### Use the `Field` type

Since the native type of values in circuits are `Field`s, using them for variables in Noir means less gates converting them under the hood.

:::tip
Where possible, use `Field` type for values. Using smaller value types, and bitpacking strategies, will result in MORE gates
:::

**Note:** Need to remain mindful of overflow. Types with less bits may be used to limit the range of possible values prior to a calculation.

### Use Arithmetic over non-arithmetic operations

Since circuits are made of arithmetic gates, the cost of them tends to be one gate. Whereas for procedural code, they represent several clock cycles.

Inversely, non-arithmetic operators are achieved with multipled gates, vs 1 clock cycle for procedural code.

| (cost\op)  | arithmetic<br>(eg `*`, `+`) | bit-wise ops<br>(eg `<`, `\|`, `>>`) |
| - | - | - |
| **cycles** | 10+ | 1 |
| **gates**  | 1 | 10+ |

:::tip
Preference arithmetic operators where possible. Attempting to optimise a circuit with bit-wise operations will lead to MORE gates.
:::

### Use static over dynamic values

Another general theme that manifests in different ways is that static reads are represented with less gates than dynamic ones.

Reading from read-only memory (ROM) adds less gates than random-access memory (RAM), 2 vs ~3.25 due to the additional bounds checks. Arrays of fixed length (albeit used at a lower capacity), will generate less gates than dynamic storage.

Related to this, if an index used to access an array is not known at compile time (ie unknown until run time), then ROM will be converted to RAM, expanding the gate count.

:::tip
Use arrays and indices that are known at compile time where possible.
NB: Using `assert_constant(i);` before an index, `i`, is used in an array will give a compile error if `i` is NOT known at compile time.
:::

### Leverage unconstrained execution

Constrained verification can leverage unconstrained execution.
Compute result via unconstrained function, verify result.

Use `  if is_unconstrained() { /`, to conditionally execute code if being called in an unconstrained vs constrained way.

#### Token transfer example - `fn sub` inefficiencies

Iterates through (32) notes, and does nullifier and membership checks...

Use unconstrained function to find two (largest) notes, if not enough recursively combine two notes.
"call" expensive, can use "multicall".

### A circuit holds all logic

In procedural execution, the following logic: `if (a>0 && b>0) {` , will not perform the second comparison if the first one is false. Whereas a circuit will hold both paths.
Implementing this type of short-circuiting is much less efficient for circuits since it is effectively adding additional comparisons which add more gates.

:::tip
Use bit-wise `&` or `|` to combine logical expressions efficiently.
:::

## Advanced

### Combine arithmetic operations

A Noir program can be honed further by combining arithmetic operators in a way that makes the most of each constraint of the backend. This is in scenarios where the backend might not be doing this perfectly.

Eg Barretenberg backend (current default for Noir) is a width-4 PLONKish constraint system
$ w_1*w_2*q_m + w_1*q_1 + w_2*q_2 + w_3*q_3 + w_4*q_4 + q_c $

Here we see there is one occurance of witness 1 and 2 ($w_1$, $w_2$) being multiplied together, with addition to witnesses 1-4 ($w_1$ .. $w_4$) multiplied by 4 corresponding circuit constants ($q_1$ .. $q_4$) (plus a final circuit constant, $q_c$).

Use `nargo info --print-acir`, to inspect the constraints, and it may present opportunities to amend the order of operations and reduce the number of constraints.

#### Variable as witness vs expression

`std::as_witness` means variable is interpreted as a witness not an expression.
When used incorrecty will create **less** efficient circuits (higher gate count).

## Rust to Noir

A few things to do when converting Rust code to Noir:
- Early `return` in function. Use `constrain` instead.
- Reference `&` operator. Remove, will be value copy.
- Type `usize`. Use types `u8`, `u32`, `u64`, ... 
- `main` return must be public, `pub`. 


## References
### Guillaume's "Cryptdoku" [video](https://www.youtube.com/watch?v=MrQyzuogxgg) (June'23):

Notes from the video:
- ROM lookup is cheaper than RAM lookup
	- 2 gates for ROM vs 3.25 for RAM due to range check (from slack)
- Fixed indices vs dynamic
- Arithmetic operations over bitwise logic and comparisons
- Fields better
- Compilation from code to circuit (constraints in a field), all values must be a field in the circuit
- If summing fields, risk overflow, can sum n-bit numbers with n and number of additions guaranteed not to overflow. But priority is use fields wherever possible.

1527 ACIR gates generated for solving sudoku

Q: Any existing guidance in changing a Rust program to Noir?
A: No.


### Tom's Tips (Jun'24):

```
- Try to avoid mutating arrays at unknown (until runtime) indices as this turns ROM into RAM which is more expensive. In this case it's best to construct the final output in an unconstrained function and then assert that it's correct.
- Unconstrained gud, so also useful in non-array settings if you can prove the result cheaply once you know it.
- Bitwise operations are bad and should be avoided if possible (notable as devs tend to use bitwise ops in an attempt to optimise their code)
- We do as much compile-time execution as we can so calling "expensive" functions with constant arguments isn't a major concern and developers shouldn't feel the need to create hardcoded constants.
```
+ Tip from Jake: `One way to avoid accessing arrays with runtime indices is putting a `assert_constant(my_index);` on the line before an array access so that you get a compile-time error if it is not constant`
### Idiomatic Noir (from vlayer)

 [aritcle](https://www.vlayer.xyz/blog/idiomatic-noir-part-1-collections)

### Tips and Tricks from Zac:

1. Compute and Measure
2. If loops are producing non-linear costs, investigate!
3. Compute in unconstrained, validate in constrained functions
  - Take care to not create invalid constraints
4. Optimise for happy path
5. If statements in loops where predicate not known at compile time are dangerous!
