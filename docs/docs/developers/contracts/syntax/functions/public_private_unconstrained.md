---
title: Public, Private, and Unconstrained Functions
---

This page explains the three types of functions that exist on Aztec - public, private, and unconstrained. For a deeper dive into how these functions work under the hood, check out the [Inner Workings](./inner_workings.md) page.

## `Public` Functions

A public function is executed by the sequencer and has access to a state model that is very similar to that of the EVM and Ethereum. Even though they work in an EVM-like model for public transactions, they are able to write data into private storage that can be consumed later by a private function.

:::note
All data inserted into private storage from a public function will be publicly viewable (not private).
:::

To create a public function you can annotate it with the `#[aztec(public)]` attribute. This will make the [public context](../context.md) available within the function's execution scope.

#include_code set_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

## `Private` Functions

A private function operates on private information, and is executed by the user. Annotate the function with the `#[aztec(private)]` attribute to tell the compiler it's a private function. This will make the [private context](../context.md#private-context-broken-down) available within the function's execution scope.

#include_code redeem_shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

## `unconstrained` functions

Unconstrained functions are an underlying part of Noir - a deeper explanation can be found [here](https://noir-lang.org/docs/noir/syntax/unconstrained). In short, they are functions which are not directly constrained and therefore should be seen as un-trusted. That they are un-trusted means that the developer must make sure to constrain their return values when used. Note: Calling an unconstrained function from a private function means that you are injecting unconstrained values.

Beyond using them inside your other functions, they are convenient for providing an interface that reads storage, applies logic and returns values to a UI or test. Below is a snippet from exposing the `balance_of_private` function from a token implementation, which allows a user to easily read their balance, similar to the `balanceOf` function in the ERC20 standard.

#include_code balance_of_private /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

:::info
Note, that unconstrained functions can have access to both public and private data when executed on the user's device. This is possible since it is not actually part of the circuits that are executed in contract execution.
:::
