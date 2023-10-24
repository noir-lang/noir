---
title: Working with TypeScript
description:
  Learn how to interact with Noir programs using TypeScript. Follow this tutorial to compile your
  program, specify inputs, initialize a prover & verifier, and prove and verify your program.
keywords: [TypeScript, Noir, tutorial, compile, inputs, prover, verifier, proof]
---

Interactions with Noir programs can also be performed in TypeScript, which can come in handy when
writing tests or when working in TypeScript-based projects like [Hardhat](https://hardhat.org/).

This guide is based on the [noir-starter](https://github.com/signorecello/noir-starter) example.
Please refer to it for an example implementation.

:::note

You may find unexpected errors working with some frameworks such as `vite`. This is due to the
nature of `wasm` files and the way Noir uses web workers. As we figure it out, we suggest using
[Create React App](https://create-react-app.dev/), or [Next.js](https://nextjs.org/) for a quick
start.

:::

## Setup

We're assuming you're using ES6 for both browser (for example with React), or nodejs.

Install [Yarn](https://yarnpkg.com/) or [Node.js](https://nodejs.org/en). Init a new project with
`npm init`. Install Noir dependencies in your project by running:

```bash
npm i @noir-lang/noir_wasm@0.3.2-fa0e9cff github:noir-lang/barretenberg#39a1547875f941ef6640217a42d8f34972425c97 @noir-lang/aztec_backend@0.1.0-0c3b2f2
```

:::note

While Noir is in rapid development, some packages could interfere with others. For that reason, you
should use these specified versions. Let us know if for some reason you need to use other ones.

:::

As for the circuit, we will use the _Standard Noir Example_ and place it in the `src` folder. Feel
free to use any other, as long as you refactor the below examples accordingly.

This standard example is a program that multiplies input `x` with input `y` and returns the result:

```rust
// src/main.nr
fn main(x: u32, y: pub u32) -> pub u32 {
    let z = x * y;
    z
}
```

One valid scenario for proving could be `x = 3`, `y = 4` and `return = 12`

## Imports

We need some imports, for both the `noir_wasm` library (which will compile the circuit into `wasm`
executables) and `aztec_backend` which is the actual proving backend we will be using.

We also need to tell the compiler where to find the `.nr` files, so we need to import
`initialiseResolver`.

```ts
import initNoirWasm, { acir_read_bytes, compile } from '@noir-lang/noir_wasm';
import initialiseAztecBackend from '@noir-lang/aztec_backend';
import { initialiseResolver } from '@noir-lang/noir-source-resolver';
```

## Compiling

We'll go over the code line-by-line later:

```ts
export const compileCircuit = async () => {
  await initNoirWasm();

  return await fetch(new URL('../src/main.nr', import.meta.url))
    .then(r => r.text())
    .then(code => {
      initialiseResolver((id: any) => {
        return code;
      });
    })
    .then(() => {
      try {
        const compiled_noir = compile({});
        return compiled_noir;
      } catch (e) {
        console.log('Error while compiling:', e);
      }
    });
};
```

1. First we're calling `initNoirWasm`. This is required on the browser only.
2. We then pass an URL that points to our `main.nr` file, and call `.then` on it so we can get the
   actual text of the source code
3. We call `initialiseResolver` returning the source code
4. Finally, we can call the `compile` function

This function should return us the compiled circuit.

:::note

You can use as many files as you need,
[importing them as you would do with Nargo](./modules_packages_crates/dependencies), and you don't
need to set them up in the `src` folder. Just mind the following particularities about
`initialiseResolver`:

1. The `compile` function expects a `main.nr` file as an entry point. If you need another one, just
   pass it as a `entry_point` parameter to `compile`. Check the
   [noir starter](https://github.com/signorecello/noir-starter) for an example on multiple files and
   a non-default entry point.
2. `initialiseResolver` needs to be synchronous
3. Different frameworks use different ways of fetching files. It's beyond the scope of this guide to
   explain why and how, but for reference,
   [noir starter](https://github.com/signorecello/noir-starter) uses both Next.js and node.js for
   testing.

Quick tip: an easy way to deal with `initialiseResolver` is just to prepare a
`{fileName: "literally_the_code"}` object beforehand 

:::

## ACIR

Noir compiles to two properties:

1. The ACIR, which is the intermediate language used by backends such as Barretenberg
2. The ABI, which tells you which inputs are to be read

Let's write a little function that gets us both, initializes the backend, and returns the ACIR as
bytes:

```ts
export const getAcir = async () => {
  const { circuit, abi } = await compileCircuit();
  await initialiseAztecBackend();

  let acir_bytes = new Uint8Array(Buffer.from(circuit, 'hex'));
  return acir_read_bytes(acir_bytes);
};
```

Calling `getAcir()` now should return us the ACIR of the circuit, ready to be used in proofs.

## Initializing Prover & Verifier

Prior to proving and verifying, the prover and verifier have to first be initialized by calling
`barretenberg`'s `setup_generic_prover_and_verifier` with your Noir program's ACIR:

```ts
let [prover, verifier] = await setup_generic_prover_and_verifier(acir);
```

This is probably a good time to store this prover and verifier into your state like React Context,
Redux, or others.

## Proving

The Noir program can then be executed and proved by calling `barretenberg`'s `create_proof`
function:

```ts
const proof = await create_proof(prover, acir, abi);
```

On the browser, this proof can fail as it requires heavy loads to be run on worker threads. Here's a
quick example of a worker:

```ts
// worker.ts
onmessage = async event => {
  try {
    await initializeAztecBackend();
    const { acir, input } = event.data;
    const [prover, verifier] = await setup_generic_prover_and_verifier(acir);
    const proof = await create_proof(prover, acir, input);
    postMessage(proof);
  } catch (er) {
    postMessage(er);
  } finally {
    close();
  }
};
```

Which would be called like this, for example:

```ts
// index.ts
const worker = new Worker(new URL('./worker.ts', import.meta.url));
worker.onmessage = e => {
  if (e.data instanceof Error) {
    // oh no!
  } else {
    // yey!
  }
};
worker.postMessage({ acir, input: { x: 3, y: 4 } });
```

## Verifying

The `proof` obtained can be verified by calling `barretenberg`'s `verify_proof` function:

```ts
// 1_mul.ts
const verified = await verify_proof(verifier, proof);
```

The function should return `true` if the entire process is working as intended, which can be
asserted if you are writing a test script:

```ts
expect(verified).eq(true);
```

## Verifying with Smart Contract

Alternatively, a verifier smart contract can be generated and used for verifying Noir proofs in
TypeScript as well.

This could be useful if the Noir program is designed to be decentrally verified and/or make use of
decentralized states and logics that is handled at the smart contract level.

To generate the verifier smart contract using typescript:

```ts
// generator.ts
import { writeFileSync } from 'fs';

const sc = verifier.SmartContract();
syncWriteFile('../contracts/plonk_vk.sol', sc);

function syncWriteFile(filename: string, data: any) {
  writeFileSync(join(__dirname, filename), data, {
    flag: 'w',
  });
}
```

You can then verify a Noir proof using the verifier contract, for example using Hardhat:

```ts
// verifier.ts
import { ethers } from 'hardhat';
import { Contract, ContractFactory, utils } from 'ethers';

let Verifier: ContractFactory;
let verifierContract: Contract;

before(async () => {
  Verifier = await ethers.getContractFactory('TurboVerifier');
  verifierContract = await Verifier.deploy();
});

// Verify proof
const sc_verified = await verifierContract.verify(proof);
```
