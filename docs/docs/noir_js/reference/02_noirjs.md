---
title: Noir
description: Reference to noir_js library and the Noir class
keywords: [Noir project, javascript, typescript, node.js, browser, react, class, reference]
---

## Table of Contents

- [constructor](#constructor)
- [init](#init)
- [generateFinalProof](#generatefinalproof)
- [verifyFinalProof](#verifyfinalproof)

## `constructor`

The `constructor` is a method used to create and initialize objects created within the `Noir` class. In the `Noir` class constructor, you need to pass two parameters: `circuit` and `backend`.

### Syntax

```js
constructor(circuit, backend);
```

### Parameters

| Parameter | Type   | Description                                                                                                                                            |
| --------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `circuit` | Object | A circuit represented in a `json` format, containing the ABI and bytecode. Typically obtained by running [`nargo compile`](../../nargo/01_commands.md) |
| `backend` | Object | A backend instance, before initialization.                                                                                                             |

### Usage

```js
const noir = new Noir(circuit, backend);
```

## `init`

This async method should be called after class instantiation. It will run processes on the ACVM, instantiate your backend, etc.

### Syntax

```js
async init()
```

### Parameters

This method takes no parameters

### Usage

```js
await noirInstance.init();
```

## `generateFinalProof`

This async method generates a witness and a proof given an object as input.

### Syntax

```js
async generateFinalproof(input)
```

### Parameters

| Parameter | Type   | Description                                      |
| --------- | ------ | ------------------------------------------------ |
| `input`   | Object | An object containing the inputs to your circuit. |

### Returns

| Return value | Type                  | Description                                         |
| ------------ | --------------------- | --------------------------------------------------- |
| `proof`      | Promise <Uint8Array\> | An array with the byte representation of the proof. |

### Usage

```js
// consider the Standard Noir Example given with nargo init
const input = { x: 1, y: 2 };
noirInstance.generateProof(input);
```

## `verifyFinalProof`

This async method instantiates the verification key and verifies your proof.

### Syntax

```js
async verifyFinalProof(proof)
```

### Parameters

| Parameter | Type       | Description                                                                                   |
| --------- | ---------- | --------------------------------------------------------------------------------------------- |
| `proof`   | Uint8Array | The Uint8Array representation of your proof, usually obtained by calling `generateFinalProof` |

### Returns

| Return value | Type               | Description                                  |
| ------------ | ------------------ | -------------------------------------------- |
| `verified`   | Promise <boolean\> | A boolean for whether the proof was verified |

### Usage

```js
const proof = noirInstance.generateProof(input);
noirInstance.verifyFinalProof(proof);
```
