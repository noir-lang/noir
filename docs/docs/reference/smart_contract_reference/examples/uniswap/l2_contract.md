---
title: L2 Contracts (Aztec)
sidebar_position: 1
---

This page goes over the code in the L2 contract for Uniswap, which works alongside a [token bridge](../../../../tutorials/contract_tutorials/advanced/token_bridge/0_setup.md). 

## Main.nr

### Setup and constructor

#include_code uniswap_setup noir-projects/noir-contracts/contracts/uniswap_contract/src/main.nr rust
We just need to store the portal address for the token that we want to swap.


### Public swap

#include_code swap_public noir-projects/noir-contracts/contracts/uniswap_contract/src/main.nr rust

1. We check that `msg.sender()` has appropriate approval to call this on behalf of the sender by constructing an authwit message and checking if `from` has given the approval (read more about authwit [here](../../../../aztec/concepts/accounts/authwit.md)).
2. We fetch the underlying aztec token that needs to be swapped.
3. We transfer the user’s funds to the Uniswap contract. Like with Ethereum, the user must have provided approval to the Uniswap contract to do so. The user must provide the nonce they used in the approval for transfer, so that Uniswap can send it to the token contract, to prove it has appropriate approval.
4. Funds are added to the Uniswap contract.
5. Uniswap must exit the input tokens to L1. For this it has to approve the bridge to burn its tokens on its behalf and then actually exit the funds. We call the [`exit_to_l1_public()` method on the token bridge](../../../../tutorials/contract_tutorials/advanced/token_bridge/index.md). We use the public flow for exiting since we are operating on public state.
6. It is not enough for us to simply emit a message to withdraw the funds. We also need to emit a message to display our swap intention. If we do not do this, there is nothing stopping a third party from calling the Uniswap portal with their own parameters and consuming our message.

So the Uniswap portal (on L1) needs to know:

- The token portals for the input and output token (to withdraw the input token to L1 and later deposit the output token to L2)
- The amount of input tokens they want to swap
- The Uniswap fee tier they want to use
- The minimum output amount they can accept (for slippage protection)

The Uniswap portal must first withdraw the input tokens, then check that the swap message exists in the outbox, execute the swap, and then call the output token to deposit the swapped tokens to L2. So the Uniswap portal must also be pass any parameters needed to complete the deposit of swapped tokens to L2. From the tutorial on building token bridges we know these are:

- The address on L2 which must receive the output tokens (remember this is public flow)
- The secret hash for consume the L1 to L2 message. Since this is the public flow the preimage doesn’t need to be a secret.

You can find the corresponding function on the [L1 contracts page](l1_contract.md).

### Private swap

#include_code swap_private noir-projects/noir-contracts/contracts/uniswap_contract/src/main.nr rust

This uses a util function `compute_swap_private_content_hash()` - find that [here](#utils)

This flow works similarly to the public flow with a few notable changes:

- Notice how in the `swap_private()`, user has to pass in `token` address which they didn't in the public flow? Since `swap_private()` is a private method, it can't read what token is publicly stored on the token bridge, so instead the user passes a token address, and `_assert_token_is_same()` checks that this user provided address is same as the one in storage. Note that because public functions are executed by the sequencer while private methods are executed locally, all public calls are always done after all private calls are done. So first the burn would happen and only later the sequencer asserts that the token is same. Note that the sequencer just sees a request to `execute_assert_token_is_same` and therefore has no context on what the appropriate private method was. If the assertion fails, then the kernel circuit will fail to create a proof and hence the transaction will be dropped.
- In the public flow, the user calls `transfer_public()`. Here instead, the user calls `unshield()`. Why? The user can't directly transfer their private tokens (their notes) to the uniswap contract, because later the Uniswap contract has to approve the bridge to burn these notes and withdraw to L1. The authwit flow for the private domain requires a signature from the `sender`, which in this case would be the Uniswap contract. For the contract to sign, it would need a private key associated to it. But who would operate this key?
- To work around this, the user can unshield their private tokens into Uniswap L2 contract. Unshielding would convert user's private notes to public balance. It is a private method on the token contract that reduces a user’s private balance and then calls a public method to increase the recipient’s (ie Uniswap) public balance. **Remember that first all private methods are executed and then later all public methods will be - so the Uniswap contract won’t have the funds until public execution begins.**
- Now uniswap has public balance (like with the public flow). Hence, `swap_private()` calls the internal public method which approves the input token bridge to burn Uniswap’s tokens and calls `exit_to_l1_public` to create an L2 → L1 message to exit to L1.
- Constructing the message content for swapping works exactly as the public flow except instead of specifying who would be the Aztec address that receives the swapped funds, we specify a secret hash (`secret_hash_for_redeeming_minted_notes`). Only those who know the preimage to the secret can later redeem the minted notes to themselves.

### Approve the bridge to burn this contract's funds

Both public and private swap functions call this function:

#include_code authwit_uniswap_set noir-projects/noir-contracts/contracts/uniswap_contract/src/main.nr rust

### Assertions

#include_code assert_token_is_same noir-projects/noir-contracts/contracts/uniswap_contract/src/main.nr rust

This is a simple function that asserts that the token passed in to the function is the one that the bridge is associated with.

## Utils

### Compute content hash for public

#include_code uniswap_public_content_hash noir-projects/noir-contracts/contracts/uniswap_contract/src/util.nr rust

This method computes the L2 to L1 message content hash for the public. To find out how it is consumed on L1, view the [L1 contracts page](./l1_contract.md)

### Compute content hash for private

#include_code compute_swap_private_content_hash noir-projects/noir-contracts/contracts/uniswap_contract/src/util.nr rust

This method computes the L2 to L1 message content hash for the private. To find out how it is consumed on L1, view the [L1 contracts page](./l1_contract.md).

## Redeeming assets

So you emitted a message to withdraw input tokens to L1 and a message to swap. Then you or someone on your behalf can swap on L1 and emit a message to deposit swapped assets to L2.

You still need to "claim" these swapped funds on L2.

In the public flow, you can call [`claim_public()`](../../../../tutorials/contract_tutorials/advanced/token_bridge/2_minting_on_aztec.md) on the output token bridge which consumes the deposit message and mints your assets.

In the private flow, you can choose to leak your secret for L1 → L2 message consumption to let someone mint the notes on L2 (by calling [`claim_private()`](../../../../tutorials/contract_tutorials/advanced/token_bridge/2_minting_on_aztec.md) on the output token bridge) and then you can later redeem these notes to yourself by presenting the preimage to `secret_hash_for_redeeming_minted_notes` and calling the `redeem_shield()` method on the token contract.