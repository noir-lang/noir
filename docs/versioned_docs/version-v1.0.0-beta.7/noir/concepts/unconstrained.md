---
title: Unconstrained Functions
description: "Learn about what unconstrained functions in Noir are, how to use them and when you'd want to."

keywords: [Noir programming language, unconstrained, brillig]
sidebar_position: 5
---

Unconstrained functions are functions which do not constrain any of the included computation and allow for non-deterministic computation.

## Why?

Zero-knowledge (ZK) domain-specific languages (DSL) enable developers to generate ZK proofs from their programs by compiling code down to the constraints of an NP complete language (such as R1CS or PLONKish languages). However, the hard bounds of a constraint system can be very limiting to the functionality of a ZK DSL.

Enabling a circuit language to perform unconstrained execution is a powerful tool. Said another way, unconstrained execution lets developers generate witnesses from code that does not generate any constraints. Being able to execute logic outside of a circuit is critical for both circuit performance and constructing proofs on information that is external to a circuit.

Fetching information from somewhere external to a circuit can also be used to enable developers to improve circuit efficiency.

A ZK DSL does not just prove computation, but proves that some computation was handled correctly. Thus, it is necessary that when we switch from performing some operation directly inside of a circuit to inside of an unconstrained environment that the appropriate constraints are still laid down elsewhere in the circuit.

## Example

An in depth example might help drive the point home. Let's look at how we can optimize a function to turn a `u72` into an array of `u8`s.

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

A lot of the operations in this function are optimized away by the compiler (all the bit-shifts turn into divisions by constants). However we can save a bunch of gates by casting to u8 a bit earlier. This automatically truncates the bit-shifted value to fit in a u8 which allows us to remove the AND against 0xff. This saves us ~480 gates in total.

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

We can then run `u72_to_u8` as unconstrained brillig code in order to calculate out, then use that result in our constrained function and assert that if we were to do the reverse calculation we'd get back num. This looks a little like the below:

```rust
fn main(num: u72) -> pub [u8; 8] {
    // Safety: 'out' is properly constrained below in 'assert(num == reconstructed_num);'
    let out = unsafe { u72_to_u8(num) };

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

Note that in order to invoke unconstrained functions we need to wrap them in an `unsafe` block,
to make it clear that the call is unconstrained.
Furthermore, a warning is emitted unless the `unsafe` block is commented with a `// Safety: ...` comment explaining why it is fine to call the unconstrained function. Note that either the `unsafe` block can be commented this way or the statement it exists in (like in the `let` example above).

Generally we want to use brillig whenever there's something that's easy to verify but hard to compute within the circuit. For example, if you wanted to calculate a square root of a number it'll be a much better idea to calculate this in brillig and then assert that if you square the result you get back your number.

## Break and Continue

In addition to loops over runtime bounds, `break` and `continue` are also available in unconstrained code. See [break and continue](../concepts/control_flow.md#break-and-continue)

## Security checks

Two compilation security passes exist currently to ensure soundness of compiled code. Problems they catch are reported as "bugs" (as opposed to errors) in the compiler output. For example:

```
**bug**: Brillig function call isn't properly covered by a manual constraint
```

### Independent subgraph detection

This pass examines the instruction flow graph to see if the final function would involve values that don't come from any provided inputs and don't result in the outputs. That would mean there are no constraints ensuring the required continuity.

This check is enabled by default and can be disabled by passing the `--skip-underconstrained-check` option to `nargo`.

### Brillig manual constraint coverage

The results of a Brillig function call must be constrained to ensure security, adhering to these rules: every resulting value (including every array element of a resulting array) has to be involved in a later constraint (i.e. assert, range check) against either one of the arguments of the call, or a constant. In this context, involvement means that a descendant value (e.g. a result of a chain of operations over the value) of a result has to be checked against a descendant value of an argument. For example:

```rust
unconstrained fn factor(v0: Field) -> [Field; 2] {
    ...
}

fn main f0 (foo: Field) -> [Field; 2] {
    let factored = unsafe { factor(foo) };
    assert(factored[0] * factored[1] == foo);
    return factored
}
```

Here, the results of `factor` are two elements of the returned array. The value `factored[0] * factored[1]` is a descendant of both of them, so both are involved in a constraint against the argument value in the `assert`. Hence, the call to an unconstrained function is properly covered.

This pass checks if the constraint coverage of Brillig calls is sufficient in these terms.

The check is enabled by default and can be disabled by passing the `--skip-brillig-constraints-check` option to `nargo`.

#### Lookback option

Certain false positives of this check can be avoided by providing the `--enable-brillig-constraints-check-lookback` option to `nargo`, which can be slower at compile-time but additionally ensures that descendants of call argument values coming from operations *preceding* the call itself would be followed. For example, consider this case:

```rust
unconstrained fn unconstrained_add(v0: Field, v1: Field) -> Field {
    v0 + v1
}

fn main f0 (v0: Field, v1: Field) {
    let foo = v0 + v1;
    let bar = unsafe { unconstrained_add(v0, v1) };
    assert(foo == bar);
    return bar
}
```

Normally, the addition operation over `v0` and `v1` happening before the call itself would prevent the call from being (correctly) considered properly constrained. With this option enabled, the false positive goes away at the cost of the check becoming somewhat less performant on large unrolled loops.
