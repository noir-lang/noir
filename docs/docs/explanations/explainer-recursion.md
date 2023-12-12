# Recursion Explainer

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

While n is not n, this function will keep calling itself until it hits the base case, bubbling up the result on the call stack.

In ZK, recursion has some similarities. It is not a Noir function calling itself, but a proof being used as an input to another circuit.

## Examples

Let's look at some examples.

### Alice and Bob - Guessing game

Alice and Bob are friends, and they like guessing games. They want to play a guessing game online, but for that, they need an trusted third-party that knows both of their secrets and finishes the game once someone wins.

So, they use zero-knowledge proofs. Alice tries to guess Bob's number, and Bob will generate a ZK proof stating whether she succeeded or failed. This ZK proof can go on a smart contract, revealing the winner and even giving prizes.

However, this means every turn needs to be verified on-chain. This incurs in some cost and waiting times.

So, Alice started thinking: "what if Bob generates his proof, and instead of sending it on-chain, I verify it *within* my own proof before playing my own turn?".

### Charlie - Recursive merkle tree

Charlie is a concerned citizen, and wants to be sure his vote in an election is accounted for. He votes with a ZK proof, but he has no way of knowing that ZK proof was included in the total vote count!

So, the tallier puts all the votes in a merkle tree, so everyone can see their own address and verify their own proof. He then proves two proofs within one proof, as such:

```
                    abcd
           __________|______________
          |                         |
         ab                         cd 
     _____|_____              ______|______
    |           |            |             |              
  alice        bob        charlie        daniel 
```

By verifying the final proof `abcd` on-chain, everyone can be sure their vote was included in the final count.

## What params do I need

As you can see in the [recursion reference](../standard_library/recursion.md), a simple recursive proof requires:

- The proof to verify
- The Verification Key of the circuit that generated the proof
- A hash of this verification key, as it's needed for some backends
- The public inputs for the proof
- The input aggregation object

It also returns the `output aggregation object`.

Two of these parameters deserve some explanation. The first one is the `verification key`, which is covered in [another explainer](../explainer-vk.md). The other one is the `aggregation object`.

### Aggregation object

Recursive zkSNARK schemes do not necessarily "verify a proof" in the sense that you expect a true or false to be spit out by the verifier. Rather an aggregation object is built over the public inputs.

In the case of PLONK the recursive aggregation object is two G1 points (expressed as 16 witness values). The final verifier (in our case this is most often the smart contract verifier) has to be aware of this aggregation object to execute a pairing and check the validity of these points.

So, taking the example of Alice and Bob:

- Alice makes her guess. Her proof is *not* recursive: it doesn't verify any proof within it! It's just a standard `assert(x != y)` circuit
- Bob verifies Alice's proof and makes his own guess. In this circuit, he is verifying a proof, so it needs to output an `aggregation object`: he is generating a recursive proof!
- Alice verifies Bob's *recursive proof*, and uses Bob's `output aggregation object` as the `input aggregation object` in her proof... Which in turn, generates another `output aggregation object`.

One should notice that when Bob generates his first proof, he has no input aggregation object. Because he is not verifying an recursive proof, he has no `input aggregation object`. In this case, he may use zeros instead.

We can imagine the `aggregation object` as the baton in a [relay race](https://en.wikipedia.org/wiki/Relay_race). The first runner doesn't have to receive the baton from anyone else, as he/she already starts with it. But when his/her turn is over, the next runner needs to receive it, run a bit more, and pass it along.
