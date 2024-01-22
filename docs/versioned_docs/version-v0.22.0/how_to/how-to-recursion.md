---
title: How to use recursion on NoirJS
description: Learn how to implement recursion with NoirJS, a powerful tool for creating smart contracts on the EVM blockchain. This guide assumes familiarity with NoirJS, solidity verifiers, and the Barretenberg proving backend. Discover how to generate both final and intermediate proofs using `noir_js` and `backend_barretenberg`.
keywords:
  [
    "NoirJS",
    "EVM blockchain",
    "smart contracts",
    "recursion",
    "solidity verifiers",
    "Barretenberg backend",
    "noir_js",
    "backend_barretenberg",
    "intermediate proofs",
    "final proofs",
    "nargo compile",
    "json import",
    "recursive circuit",
    "recursive app"
  ]
sidebar_position: 1
---

This guide shows you how to use recursive proofs in your NoirJS app. For the sake of clarity, it is assumed that:

- You already have a NoirJS app. If you don't, please visit the [NoirJS tutorial](../tutorials/noirjs_app.md) and the [reference](../reference/NoirJS/noir_js/index.md).
- You are familiar with what are recursive proofs and you have read the [recursion explainer](../explainers/explainer-recursion.md)
- You already built a recursive circuit following [the reference](../noir/standard_library/recursion.md), and understand how it works.

It is also assumed that you're not using `noir_wasm` for compilation, and instead you've used [`nargo compile`](../reference/nargo_commands.md) to generate the `json` you're now importing into your project. However, the guide should work just the same if you're using `noir_wasm`.

:::info

As you've read in the [explainer](../explainers/explainer-recursion.md), a recursive proof is an intermediate proof. Meaning it doesn't necessarily generate the final step that makes it verifiable in a smart contract. However, it is easy to verify within another circuit.

While "standard" usage of NoirJS packages abstracts final proofs, it currently lacks the necessary interface to abstract away intermediate proofs. Which means these proofs need to be created by using the backend directly.

In short:

- `noir_js` generates *only* final proofs
- `backend_barretenberg` generates both types of proofs

:::

In a standard recursive app, you're also dealing with at least two circuits. For the purpose of this guide, we will assume these two:

- `main`: a circuit of type `assert(x != y)`
- `recursive`: a circuit that verifies `main`

For a full example on how recursive proofs work, please refer to the [noir-examples](https://github.com/noir-lang/noir-examples) repository. We will *not* be using it as a reference for this guide.

## Step 1: Setup

In a common NoirJS app, you need to instantiate a backend with something like `const backend = new Backend(circuit)`. Then you feed it to the `noir_js` interface.

For recursion, this doesn't happen, and the only need for `noir_js` is only to `execute` a circuit and get its witness and return value. Everything else is not interfaced, so it needs to happen on the `backend` object.

It is also recommended that you instantiate the backend with as many threads as possible, to allow for maximum concurrency:

```js
const backend = new Backend(circuit, { threads: 8 })
```

:::tip
You can use the [`os.cpus()`](https://nodejs.org/api/os.html#oscpus) object in `nodejs` or [`navigator.hardwareConcurrency`](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/hardwareConcurrency) on the browser to make the most out of those glorious cpu cores
:::

## Step 2: Generating the witness and the proof for `main`

After instantiating the backend, you should also instantiate `noir_js`. We will use it to execute the circuit and get the witness.

```js
const noir = new Noir(circuit, backend)
const { witness } = noir.execute(input)
```

With this witness, you are now able to generate the intermediate proof for the main circuit:

```js
const { proof, publicInputs } = await backend.generateIntermediateProof(witness)
```

:::warning

Always keep in mind what is actually happening on your development process, otherwise you'll quickly become confused about what circuit are we actually running and why!

In this case, you can imagine that Alice (running the `main` circuit) is proving something to Bob (running the `recursive` circuit), and Bob is verifying her proof within his proof.

With this in mind, it becomes clear that our intermediate proof is the one *meant to be verified within another circuit*. So it is Alice's. Actually, the only final proof in this theoretical scenario would be the last one, sent on-chain.

:::

## Step 3 - Verification and proof artifacts

Optionally, you are able to verify the intermediate proof:

```js
const verified = await backend.verifyIntermediateProof({ proof, publicInputs })
```

This can be useful to make sure our intermediate proof was correctly generated. But the real goal is to do it within another circuit. For that, we need to generate the intermediate artifacts:

```js
const { proofAsFields, vkAsFields, vkHash } = await backend.generateIntermediateProofArtifacts( { publicInputs, proof }, publicInputsCount)
```

This call takes the public inputs and the proof, but also the public inputs count. While this is easily retrievable by simply counting the `publicInputs` length, the backend interface doesn't currently abstract it away.

:::info

The `proofAsFields` has a constant size `[Field; 93]`. However, currently the backend doesn't remove the public inputs from the proof when converting it.

This means that if your `main` circuit has two public inputs, then you should also modify the recursive circuit to accept a proof with the public inputs appended. This means that in our example, since `y` is a public input, our `proofAsFields` is of type `[Field; 94]`.

Verification keys in Barretenberg are always of size 114.

:::

:::warning

One common mistake is to forget *who* makes this call.

In a situation where Alice is generating the `main` proof, if she generates the proof artifacts and sends them to Bob, which gladly takes them as true, this would mean Alice could prove anything!

Instead, Bob needs to make sure *he* extracts the proof artifacts, using his own instance of the `main` circuit backend. This way, Alice has to provide a valid proof for the correct `main` circuit.

:::

## Step 4 - Recursive proof generation

With the artifacts, generating a recursive proof is no different from a normal proof. You simply use the `backend` (with the recursive circuit) to generate it:

```js
const recursiveInputs = {
    verification_key: vkAsFields, // array of length 114
    proof: proofAsFields, // array of length 93 + size of public inputs
    publicInputs: [mainInput.y], // using the example above, where `y` is the only public input
    key_hash: vkHash,
    input_aggregation_object: Array(16).fill(0) // this circuit is verifying a non-recursive proof, so there's no input aggregation object: just use zero
}

const { witness, returnValue } = noir.execute(recursiveInputs) // we're executing the recursive circuit now!
const { proof, publicInputs } = backend.generateFinalProof(witness)
const verified = backend.verifyFinalProof({ proof, publicInputs })
```

You can obviously chain this proof into another proof. In fact, if you're using recursive proofs, you're probably interested of using them this way! In that case, you should keep in mind the `returnValue`, as it will contain the `input_aggregation_object` for the next proof.

:::tip

Managing circuits and "who does what" can be confusing. To make sure your naming is consistent, you can keep them in an object. For example:

```js
const circuits = {
main: mainJSON, 
recursive: recursiveJSON
}
const backends = {
main: new BarretenbergBackend(circuits.main),
recursive: new BarretenbergBackend(circuits.recursive)
}
const noir_programs = {
main: new Noir(circuits.main, backends.main),
recursive: new Noir(circuits.recursive, backends.recursive)
}
```

This allows you to neatly call exactly the method you want without conflicting names:

```js
// Alice runs this 👇
const { witness: mainWitness } = await noir_programs.main.execute(input)
const proof = await backends.main.generateIntermediateProof(mainWitness)

// Bob runs this 👇
const verified = await backends.main.verifyIntermediateProof(proof)
const { proofAsFields, vkAsFields, vkHash } = await backends.main.generateIntermediateProofArtifacts(
    proof,
    numPublicInputs,
);
const recursiveProof = await noir_programs.recursive.generateFinalProof(recursiveInputs)
```

:::
