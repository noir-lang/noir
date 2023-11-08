---
title: Unconstrained Functions
description: "Learn about what unconstrained functions in Noir are, how to use them and when you'd want to."

keywords: [Noir programming language, unconstrained, open]
---

<!-- Source material for much of this page comes from here: https://github.com/noir-lang/acvm-docs/blob/main/docs/brillig/00_intro.md -->

Unconstrained functions are functions which do not constrain any of the included computation and allow for non-determinisitic computation.

## Why?

Zero-knowledge (ZK) domain-specific languages (DSL) enable developers to generate ZK proofs from their programs by compiling code down to the constraints of an NP complete language (such as R1CS or PLONKish languages). However, the hard bounds of a constraint system can be very limiting to the functionality of a ZK DSL.

Enabling a circuit language to perform unconstrained execution is a powerful tool. Said another way, unconstrained execution lets developers generate witnesses from code that does not generate any constraints. Being able to execute logic outside of a circuit is critical for both circuit performance and constructing proofs on information that is external to a circuit.

Fetching information from somewhere external to a circuit can also be used to enable developers to improve circuit efficiency.

A ZK DSL does not just prove computation, but proves that some computation was handled correctly. Thus, it is necessary that when we switch from performing some operation directly inside of a circuit to inside of an unconstrained environment that the appropriate constraints are still laid down elsewhere in the circuit.

## Example

An in depth example might help drive the point home. This example comes from the excellent [post](https://discord.com/channels/1113924620781883405/1124022445054111926/1128747641853972590) by Tom in the Noir Discord.

Let's look at how we can optimize a function to turn a `u72` into an array of `u8`s.

```rust
fn main(num: u72) -> pub [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i * 8)) as u72 & 0xff) as u8;
    }

    out
}
```

```
Total ACIR opcodes generated for language PLONKCSat { width: 3 }: 91
Backend circuit size: 3619
```

A lot of the operations in this function are optimized away by the compiler (all the bit-shifts turn into divisions by constants). However we can save a bunch of gates by casting to u8 a bit earlier. This automatically truncates the bit-shifted value to fit in a u8 which allows us to remove the XOR against 0xff. This saves us ~480 gates in total.

```rust
fn main(num: u72) -> pub [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i * 8)) as u8;
    }

    out
}
```

```
Total ACIR opcodes generated for language PLONKCSat { width: 3 }: 75
Backend circuit size: 3143
```

Those are some nice savings already but we can do better. This code is all constrained so we're proving every step of calculating out using num, but we don't actually care about how we calculate this, just that it's correct. This is where brillig comes in.

It turns out that truncating a u72 into a u8 is hard to do inside a snark, each time we do as u8 we lay down 4 ACIR opcodes which get converted into multiple gates. It's actually much easier to calculate num from out than the other way around. All we need to do is multiply each element of out by a constant and add them all together, both relatively easy operations inside a snark.

We can then run u72_to_u8 as unconstrained brillig code in order to calculate out, then use that result in our constrained function and assert that if we were to do the reverse calculation we'd get back num. This looks a little like the below:

```rust
fn main(num: u72) -> pub [u8; 8] {
    let out = u72_to_u8(num);

    let mut reconstructed_num: u72 = 0;
    for i in 0..8 {
        reconstructed_num += (out[i] as u72 << (56 - (8 * i)));
    }
    assert(num == reconstructed_num);
    out
}

unconstrained fn u72_to_u8(num: u72) -> [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i * 8))) as u8;
    }
    out
}
```

```
Total ACIR opcodes generated for language PLONKCSat { width: 3 }: 78
Backend circuit size: 2902
```

This ends up taking off another ~250 gates from our circuit! We've ended up with more ACIR opcodes than before but they're easier for the backend to prove (resulting in fewer gates).

Generally we want to use brillig whenever there's something that's easy to verify but hard to compute within the circuit. For example, if you wanted to calculate a square root of a number it'll be a much better idea to calculate this in brillig and then assert that if you square the result you get back your number.
