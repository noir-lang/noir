# Writing an Account Contract

This tutorial will take you through the process of writing your own account contract in Noir, along with the Typescript glue code required for using it within a [wallet](./main.md).

Writing your own account contract allows you to define the rules by which user transactions are authorised and paid for, as well as how user keys are managed (including key rotation and recovery). In other words, writing an account contract lets you make the most out of [account abstraction](../../concepts/foundation/accounts/main.md#what-is-account-abstraction) in the Aztec network.

It is highly recommended that you understand how an [account](../../concepts/foundation/accounts/main.md) is defined in Aztec, as well as the differences between privacy and authentication [keys](../../concepts/foundation/accounts/keys.md). You will also need to know how to write a [contract in Noir](../contracts/main.md), as well as some basic [Typescript](https://www.typescriptlang.org/).

For this tutorial, we will write an account contract that uses Schnorr signatures for authenticating transaction requests.

> That is, every time a transaction payload is passed to this account contract's 'entrypoint' function, the account contract will demand a valid Schnorr signature, whose signed message matches the transaction payload, and whose signer matches the account contract owner's public key. If the signature fails, the transaction will fail.

For the sake of simplicity, we will hardcode the signing public key into the contract, but you could store it [in a private note](../../concepts/foundation/accounts/keys.md#using-a-private-note), [in an immutable note](../../concepts/foundation/accounts/keys.md#using-an-immutable-private-note), or [on a separate keystore](../../concepts/foundation/accounts/keys.md#using-a-separate-keystore), to mention a few examples.

## The account contract

Let's start with the account contract itself in Noir. Create [a new Noir contract project](../contracts/main.md) that will contain a file with the code for the account contract, with a hardcoded public key:

#include_code contract yarn-project/noir-contracts/src/contracts/schnorr_hardcoded_account_contract/src/main.nr rust

:::info
You can use [the Aztec CLI](../cli/main.md) to generate a new keypair if you want to use a different one:
```bash
$ aztec-cli generate-private-key
```

```bash
Private Key: 0xc06461a031058f116f087bc0161b11c039648eb47e03bad3eab089709bf9b8ae
Public Key:  0x0ede151adaef1cfcc1b3e152ea39f00c5cda3f3857cef00decb049d283672dc713c0e184340407e796411f74b7383252f1406272b58fccad6fee203f8a6db474
```
:::

The important part of this contract is the `entrypoint` function, which will be the first function executed in any transaction originated from this account. This function has two main responsibilities: 1) authenticating the transaction and 2) executing calls. It receives a `payload` with the list of function calls to execute, as well as a signature over that payload.

#include_code entrypoint-struct yarn-project/noir-libs/noir-aztec/src/entrypoint.nr rust

:::info
Using the `EntrypointPayload` struct is not mandatory. You can package the instructions to be carried out by your account contract however you want. However, the entrypoint payload already provides a set of helper functions, both in Noir and Typescript, that can save you a lot of time when writing a new account contract.
:::

Let's go step by step into what the `entrypoint` function is doing:

#include_code entrypoint-auth yarn-project/noir-contracts/src/contracts/schnorr_hardcoded_account_contract/src/main.nr rust

We authenticate the transaction. To do this, we serialise and Pedersen-hash the payload, which contains the instructions to be carried out along with a nonce. We then assert that the signature verifies against the resulting hash and the contract public key. This makes a transaction with an invalid signature unprovable.

#include_code entrypoint-auth yarn-project/noir-contracts/src/contracts/schnorr_hardcoded_account_contract/src/main.nr rust

Last, we execute the calls in the payload struct. The `execute_calls` helper function runs through the private and public calls included in the entrypoint payload and executes them:

#include_code entrypoint-execute-calls yarn-project/noir-libs/noir-aztec/src/entrypoint.nr rust

Note the usage of the `_with_packed_args` variant of [`call_public_function` and `call_private_function`](../contracts/functions.md#calling-functions). Due to Noir limitations, we cannot include more than a small number of arguments in a function call. However, we can bypass this restriction by using a hash of the arguments in a function call, which gets automatically expanded to the full set of arguments when the nested call is executed. We call this _argument packing_.

## The typescript side of things

Now that we have a valid Noir account contract, we need to write the typescript glue code that will take care of formatting and authenticating transactions so they can be processed by our contract, as well as deploying the contract during account setup. This takes the form of implementing the `AccountContract` interface:

#include_code account-contract-interface yarn-project/aztec.js/src/account/contract/index.ts typescript

The most interesting bit here is creating an `Entrypoint`, which is the piece of code that converts from a list of function calls requested by the user into a transaction execution request that can be simulated and proven:

#include_code entrypoint-interface yarn-project/aztec.js/src/account/entrypoint/index.ts typescript

For our account contract, we need to assemble the function calls into a payload, sign it using Schnorr, and encode these arguments for our `entrypoint` function. Let's see how it would look like:

#include_code account-contract yarn-project/end-to-end/src/guides/writing_an_account_contract.test.ts typescript

Note that we are using the `buildPayload` and `hashPayload` helpers for assembling and Pedersen-hashing the `EntrypointPayload` struct for our `entrypoint` function. As mentioned, this is not required, and you can define your own structure for instructing your account contract what functions to run.

Then, we are using the `Schnorr` signer from the `@aztec/circuits.js` package to sign over the payload hash. This signer maps to exactly the same signing scheme that Noir's standard library expects in `schnorr::verify_signature`. 

:::info
More signing schemes are available in case you want to experiment with other types of keys. Check out Noir's [documentation on cryptographic primitives](https://noir-lang.org/standard_library/cryptographic_primitives).
:::

Last, we use the `buildTxExecutionRequest` helper function to assemble the transaction execution request from the arguments and entrypoint. Note that we are also including the set of packed arguments that map to each of the nested calls: these are required for unpacking the arguments in functions calls executed via `_with_packed_args`.

## Trying it out

Let's try creating a new account backed by our account contract, and interact with a simple token contract to test it works.

<!-- TODO: Link to docs showing how to get an instance of Aztec RPC server  -->
To create and deploy the account, we will use the `Account` class, which takes an instance of an Aztec RPC server, a [privacy private key](../../concepts/foundation/accounts/keys.md#privacy-keys), and an instance of our `AccountContract` class:

#include_code account-contract-deploy yarn-project/end-to-end/src/guides/writing_an_account_contract.test.ts typescript

Note that we get a [`Wallet` instance](./main.md) out of the account, which we can use for initialising the token contract class after deployment, so any transactions sent to it are sent from our wallet. We can then send a transaction to it and check its effects:

#include_code account-contract-works yarn-project/end-to-end/src/guides/writing_an_account_contract.test.ts typescript

If we run this, we get `Balance of wallet is now 150`, which shows that the `mint` call was successfully executed from our account contract.

To make sure that we are actually validating the provided signature in our account contract, we can try signing with a different key. To do this, we will set up a new `Account` instance pointing to the contract we already deployed but using a wrong signing key:

#include_code account-contract-fails yarn-project/end-to-end/src/guides/writing_an_account_contract.test.ts typescript

Lo and behold, we get `Error: Assertion failed: 'verification == true'` when running the snippet above, pointing to the line in our account contract where we verify the Schnorr signature.