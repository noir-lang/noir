---
title: L2 Contract Setup
---

As we mentioned in [the overview](./main.md), the Uniswap L2 contract will receive funds from the user and then burn funds on L2 to withdraw. To do this it calls `TokenBridge.exit_to_l1_public()` which burns funds on the Uniswap contract. The bridge needs approval from the Uniswap contract to burn its funds.

In this step, we will set up the storage struct for our Uniswap contract and define the functions to approve and validate burn actions.

Our main contract will live inside `uniswap/src/main.nr`. In `main.nr`, paste this initial setup code:

#include_code uniswap_setup yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust

**What’s happening here?**

Because Uniswap has to approve the bridge to withdraw funds, it has to handle the approvals. So it stores a map of all the actions that are approved. The approval message is hashed to a field and stored in the contract’s storage in the `approved_action` map.

To ensure there are no collisions (i.e. when the contract wants to approve the bridge of the exact same amount, the message hash would be the same), we also keep a nonce that gets incremented each time after use in a message.

## Building the approval flow
Next, paste this function:

#include_code authwit_uniswap_get yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust

In this function, the token contract calls the Uniswap contract to check if Uniswap has indeed done the approval. The token contract expects a `is_valid()` function to exit for private approvals and `is_valid_public()` for public approvals. If the action is indeed approved, it expects that the contract would return the function selector for `is_valid()`  in both cases. The Aztec.nr library exposes this constant for ease of use. The token contract also emits a nullifier for this message so that this approval (with the nonce) can’t be used again.

This is similar to the [Authwit flow](../../contracts/resources/common_patterns/authwit.md).

However we don't have a function that actually creates the approved message and stores the action. This method should be responsible for creating the approval and then calling the token bridge to withdraw the funds to L1:

#include_code authwit_uniswap_set yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust

Notice how the nonce also gets incremented.

In the next step we’ll go through a public swapping flow.
