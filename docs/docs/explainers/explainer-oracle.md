---
title: Oracles
description: 
keywords:
  [
  ]
sidebar_position: 1
---

If you've seen "The Matrix" you may recall "The Oracle" as Gloria Foster smoking cigarettes and baking cookies. While she appears to "know things", she is actually providing a calculation of a pre-determined future. Noir Oracles are similar, in a way. They don't calculate the future (yet), but they allow you to use outside calculations in your programs.

A Noir program is usually self-contained. You can pass certain inputs to it, and it will generate a deterministic output for those inputs. But what if you wanted to defer these calculations to an outside calculation? What if you wanted to get trusted information from an outside source?

Since Noir executes on the client-side, Oracles are functions that make JSON-RPC calls to a server and use them for their execution.

## Uses

An example usage for Oracles is proving something on-chain. For example, proving that the ETH-USDC exchange was below a certain target at a certain block time, or even making more complex proofs like proving the ownership of an NFT as an anonymous login method.

However, one could use Oracles to defer expensive calculations to be made outside of the circuit, and then constraining the result. This is no different from the example in the [unconstrained page](../noir/concepts//unconstrained.md).

In short, anything that can be constrained in a circuit but needs to be fetched from an outside source is a great candidate to be used in oracles.

## Constraining oracles

Just like in Matrix, Oracles are powerful. But with great power, comes great responsibility! Just because you're using them in a Noir program, that doesn't mean they're true. Noir has no superpowers: if you want to prove that Portugal won the Euro Cup 2016, you're still relying on the information the RPC server is giving you.

To give a concrete example, Alice wants to login to the NounsDAO forum with her username "noir_nouner" by proving she owns a noun without doxxing her ethereum address. Her Noir program could have a oracle call like this:

```rust
#[oracle(getNoun)]
unconstrained fn get_noun(address: u32) -> Field
```

This oracle would naively resolve with how many nouns she possesses. However, the oracle is *also* running locally, so it could return anything she wants. It is useless as a trusted source, even though it is a zero-knowledge circuit. In order to make this oracle call actually useful, Alice would need to constrain the response, by proving her address and the noun count belongs to the state tree of the contract.

In short, Oracles don't prove anything. Your Noir program does.

:::danger

If you don't constrain the return of your oracle, you could be clearly opening an attack vector on your Noir program. Make double-triple sure that the return of an oracle call is constrained!

:::

## Return values

As you can see above, Noir needs to know what to pass to an oracle, and what to expect from it. These types are defined as `ForeignCallParam`:

```rust
pub enum ForeignCallParam {
    Single(Value),
    Array(Vec<Value>),
}
```

This means that whatever you pass to an oracle, it needs to be either a `Single` or an `Array` of values. Although possible, for the time being there are no plans to support multidimensional arrays or more complex structures.

## One server

Noir Oracles are not composable. This means that Noir will call *one* JSON-RPC server to get values. So, if you want to build using oracles, you should also run an JSON-RPC server. Follow through to the [oracle guide](../how_to/how-to-oracles.md) for a simple example on how to do that.
