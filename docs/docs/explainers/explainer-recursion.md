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
    "Aggregation Objects",
    "Recursive zkSNARK schemes",
    "PLONK",
    "Proving and Verification Keys"
  ]
sidebar_position: 1
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

It is not a Noir function calling itself, but a proof being used as an input to another circuit. In short, you verify one proof *inside* another proof, returning the proof that both proofs were valid.

This can mean that, given enough computational resources, you can prove the correctness of any arbitrary number of proofs before, for example, settling them on-chain. This could be useful to design state channels (for which a common example would be [Bitcoin's Lightning Network](https://en.wikipedia.org/wiki/Lightning_Network)), to save on gas costs, or simply to make business logic less dependent on a consensus mechanism.

## Examples

Let us look at some of these examples

### Alice and Bob - Guessing game

Alice and Bob are friends, and they like guessing games. They want to play a guessing game online, but for that, they need an trusted third-party that knows both of their secrets and finishes the game once someone wins.

So, they use zero-knowledge proofs. Alice tries to guess Bob's number, and Bob will generate a ZK proof stating whether she succeeded or failed.

This ZK proof can go on a smart contract, revealing the winner and even giving prizes. However, this means every turn needs to be verified on-chain. This incurs in some cost and waiting times that may simply turn the game too expensive or time-consuming to be worth it.

So, Alice started thinking: "what if Bob generates his proof, and instead of sending it on-chain, I verify it *within* my own proof before playing my own turn?". She can then generate a proof that she verified his proof, and so on.

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

So, the tallier puts all the votes in a merkle tree, and everyone can also prove the verification of two proofs within one proof, as such:

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

Daniel has a big circuit and a big headache. A part of his circuit is a setup phase that finishes with some assertions that need to be made. But that section alone takes 40% (80sec) of the 200sec proving time, and is largely independent of the rest of the circuit.

He could find it more efficient to generate a proof for that setup phase separately, and verifying it his actual business logic section of the circuit. For example, at the time of writing, on a Macbook M2, a recursive proof would take about 40 seconds using all the 8 cores in parallel.

This would make Daniel save 40 seconds by leveraging recursive proofs.

## What params do I need

As you can see in the [recursion reference](noir/standard_library/recursion.md), a simple recursive proof requires:

- The proof to verify
- The Verification Key of the circuit that generated the proof
- A hash of this verification key, as it's needed for some backends
- The public inputs for the proof
- The input aggregation object

It also returns the `output aggregation object`. These aggregation objects can be confusing at times, so let's dive in a little bit.

### Aggregation objects

Recursive zkSNARK schemes do not necessarily "verify a proof" in the sense that you expect a true or false to be spit out by the verifier. Rather an aggregation object is built over the public inputs.

In the case of PLONK the recursive aggregation object is two G1 points (expressed as 16 witness values). The final verifier (in our case this is most often the smart contract verifier) has to be aware of this aggregation object to execute a pairing and check the validity of these points.

So, taking the example of Alice and Bob and their guessing game:

- Alice makes her guess. Her proof is *not* recursive: it doesn't verify any proof within it! It's just a standard `assert(x != y)` circuit
- Bob verifies Alice's proof and makes his own guess. In this circuit, he is verifying a proof, so it needs to output an `aggregation object`: he is generating a recursive proof!
- Alice verifies Bob's *recursive proof*, and uses Bob's `output aggregation object` as the `input aggregation object` in her proof... Which in turn, generates another `output aggregation object`.

One should notice that when Bob generates his first proof, he has no input aggregation object. Because he is not verifying an recursive proof, he has no `input aggregation object`. In this case, he may use zeros instead.

We can imagine the `aggregation object` as the baton in a [relay race](https://en.wikipedia.org/wiki/Relay_race). The first runner doesn't have to receive the baton from anyone else, as he/she already starts with it. But when his/her turn is over, the next runner needs to receive it, run a bit more, and pass it along. Even though every runner could theoretically verify the baton mid-run (why not? 🏃🔍), only at the end of the race does the referee verify that the whole race is valid.

## How many circuits do I need

This is a common question, as there's not a one-size-fits-all solution. Here are three out of the many possible configurations:

### Two, but watch out

A common pattern, that requires some resources, is to have everyone generate both a main proof and a recursive proof. This involves two circuits:

- A `main`, non-recursive circuit with some logic
- A `recursive` circuit meant to verify two proofs in one proof

In this case, if Alice and Bob are playing a chess game and Alice is playing whites, she would immediately start by sending Bob a recursive proof, by verifying her `main` proof twice to start with.

From then on, Bob would generate his `main` proof and verify both Alice's recursive proof, and his own `main` proof. Everyone generates two proofs, which could be intensive.

### The light approach

In some one-way interaction situations, one could shave off one of the recursive proofs. This would fit, for example, a situation where the first prover is using a phone, which is currently unfit to generate recursive proofs in an acceptable time frame. With this configuration, only the first, lighter proof, is generated on the phone.

- A `main`, non-recursive circuit with some logic
- A `first proof` recursive circuit meant to verify only the first `main` circuit
- A `recursive` circuit meant to verify two proofs in one proof

To give a practical example, a barman wouldn't need to verify a "proof-of-age" on-chain every time he serves alcohol to a customer. These proofs would be made on the customer's phones, and the barman would just verify them locally, aggregating them into a final proof sent on-chain at the end of the day. However, the very first customer needs its own circuit, as there's only one proof to verify.

### Different logic

Nothing prevents you from verifying different circuits in a recursive proof, for example:

- A `circuit1` circuit
- A `circuit2` circuit
- A `recursive` circuit

To give another practical example, a regulator could verify that taxes were paid for a specific purchase by aggregating both a `payer` circuit (proving that a purchase was made and taxes were paid), and a `receipt` circuit (proving that the payment was received)

## How fast is it

At the time of writing, verifying recursive proofs is surprisingly fast. This is because most of the time is spent on generating the verification key that will be used to generate the next proof. So you are able to cache the verification key and reuse it later.

Currently, Noir JS packages don't expose the functionality of loading proving and verification keys, but that feature exists in the underlying `bb.js` package.
