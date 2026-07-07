---
title: Recursive proofs
description: Explore the concept of recursive proofs in Zero-Knowledge programming. Understand how recursion works in Noir, a language for writing smart contracts on the EVM blockchain. Learn through practical examples like Alice and Bob's guessing game, Charlie's recursive merkle tree, and Daniel's reusable components. Discover how to use recursive proofs to optimize computational resources and improve efficiency.

keywords:
  [
    "Recursive Proofs",
    "Zero-Knowledge Programming",
    "Noir",
    "EVM Blockchain",
    "Smart Contracts",
    "Recursion in Noir",
    "Alice and Bob Guessing Game",
    "Recursive Merkle Tree",
    "Reusable Components",
    "Optimizing Computational Resources",
    "Improving Efficiency",
    "Verification Key",
    "Aggregation",
    "Recursive zkSNARK schemes",
    "PLONK",
    "Proving and Verification Keys"
  ]
sidebar_position: 1
pagination_next: how_to/how-to-recursion
---

In programming, we tend to think of recursion as something calling itself. A classic example would be the calculation of the factorial of a number:

```js
function factorial(n) {
    if (n === 0 || n === 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
```

In this case, while `n` is not `1`, this function will keep calling itself until it hits the base case, bubbling up the result on the call stack:

```md
        Is `n` 1?  <---------
           /\               /
          /  \         n = n -1 
         /    \           /
       Yes     No --------
```

In Zero-Knowledge, recursion has some similarities.

It is not a Noir function calling itself, but a proof being used as an input to another circuit. In short, you verify one proof *inside* another proof, returning the proof that both proofs are valid.

This means that, given enough computational resources, you can prove the correctness of any arbitrary number of proofs in a single proof. This could be useful to design state channels (for which a common example would be [Bitcoin's Lightning Network](https://en.wikipedia.org/wiki/Lightning_Network)), to save on gas costs by settling one proof on-chain, or simply to make business logic less dependent on a consensus mechanism.

## Examples

Let us look at some of these examples

### Alice and Bob - Guessing game

Alice and Bob are friends, and they like guessing games. They want to play a guessing game online, but for that, they need a trusted third-party that knows both of their secrets and finishes the game once someone wins.

So, they use zero-knowledge proofs. Alice tries to guess Bob's number, and Bob will generate a ZK proof stating whether she succeeded or failed.

This ZK proof can go on a smart contract, revealing the winner and even giving prizes. However, this means every turn needs to be verified on-chain. This incurs some cost and waiting time that may simply make the game too expensive or time-consuming to be worth it.

As a solution, Alice proposes the following: "what if Bob generates his proof, and instead of sending it on-chain, I verify it *within* my own proof before playing my own turn?".

She can then generate a proof that she verified his proof, and so on.

```md
      Did you fail?  <--------------------------
           / \                                  /
          /   \                             n = n -1 
         /     \                              /
       Yes      No                           /
        |        |                          /
        |        |                         /
        |      You win                    /
        |                                /
        |                               /
Generate proof of that                 /
        +                             /
    my own guess     ----------------
```

### Charlie - Recursive merkle tree

Charlie is a concerned citizen, and wants to be sure his vote in an election is accounted for. He votes with a ZK proof, but he has no way of knowing that his ZK proof was included in the total vote count!

If the vote collector puts all of the votes into a [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree), everyone can prove the verification of two proofs within one proof, as such:

```md
                    abcd
           __________|______________
          |                         |
         ab                         cd 
     _____|_____              ______|______
    |           |            |             |              
  alice        bob        charlie        daniel 
```

Doing this recursively allows us to arrive on a final proof `abcd` which if true, verifies the correctness of all the votes.

### Daniel - Reusable components

Daniel has a big circuit and a big headache. A part of his circuit is a setup phase that finishes with some assertions that need to be made. But that section alone takes most of the proving time, and is largely independent of the rest of the circuit.

He might find it more efficient to generate a proof for that setup phase separately, and verify that proof recursively in the actual business logic section of his circuit. This will allow for parallelization of both proofs, which results in a considerable speedup.

## What params do I need

As you can see in the [recursion reference](noir/standard_library/recursion.mdx), a simple recursive proof requires:

- The proof to verify
- The Verification Key of the circuit that generated the proof
- A hash of this verification key, as it's needed for some backends
- The public inputs for the proof

:::info

Recursive zkSNARK schemes do not necessarily "verify a proof" in the sense that you expect a true or false to be spit out by the verifier. Rather an aggregation object is built over the public inputs.

So, taking the example of Alice and Bob and their guessing game:

- Alice makes her guess. Her proof is *not* recursive: it doesn't verify any proof within it! It's just a standard `assert(x != y)` circuit
- Bob verifies Alice's proof and makes his own guess. In this circuit, he doesn't exactly *prove* the verification of Alice's proof. Instead, he *aggregates* his proof to Alice's proof. The actual verification is done when the full proof is verified, for example when using `nargo verify` or through the verifier smart contract.

We can imagine recursive proofs a [relay race](https://en.wikipedia.org/wiki/Relay_race). The first runner doesn't have to receive the baton from anyone else, as he/she already starts with it. But when his/her turn is over, the next runner needs to receive it, run a bit more, and pass it along. Even though every runner could theoretically verify the baton mid-run (why not? 🏃🔍), only at the end of the race does the referee verify that the whole race is valid.

:::

## Some architecture

As with everything in computer science, there's no one-size-fits all. But there are some patterns that could help understanding and implementing them. To give three examples:

### Adding some logic to a proof verification

This would be an approach for something like our guessing game, where proofs are sent back and forth and are verified by each opponent. This circuit would be divided in two sections:

- A `recursive verification` section, which would be just the call to `std::verify_proof`, and that would be skipped on the first move (since there's no proof to verify)
- A `guessing` section, which is basically the logic part where the actual guessing happens

In such a situation, and assuming Alice is first, she would skip the first part and try to guess Bob's number. Bob would then verify her proof on the first section of his run, and try to guess Alice's number on the second part, and so on.

### Aggregating proofs

In some one-way interaction situations, recursion would allow for aggregation of simple proofs that don't need to be immediately verified on-chain or elsewhere.

To give a practical example, a barman wouldn't need to verify a "proof-of-age" on-chain every time he serves alcohol to a customer. Instead, the architecture would comprise two circuits:

- A `main`, non-recursive circuit with some logic
- A `recursive` circuit meant to verify two proofs in one proof

The customer's proofs would be intermediate, and made on their phones, and the barman could just verify them locally. He would then aggregate them into a final proof sent on-chain (or elsewhere) at the end of the day.

### Recursively verifying different circuits

Nothing prevents you from verifying different circuits in a recursive proof, for example:

- A `circuit1` circuit
- A `circuit2` circuit
- A `recursive` circuit

In this example, a regulator could verify that taxes were paid for a specific purchase by aggregating both a `payer` circuit (proving that a purchase was made and taxes were paid), and a `receipt` circuit (proving that the payment was received)

## How fast is it

At the time of writing, verifying recursive proofs is surprisingly fast. This is because most of the time is spent on generating the verification key that will be used to generate the next proof. So you are able to cache the verification key and reuse it later.

Currently, Noir JS packages don't expose the functionality of loading proving and verification keys, but that feature exists in the underlying `bb.js` package.

## How can I try it

Learn more about using recursion in Nargo and NoirJS in the [how-to guide](../how_to/how-to-recursion.md) and see a full example in [noir-examples](https://github.com/noir-lang/noir-examples).
