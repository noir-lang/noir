---
title: Swapping Privately
---

In the `uniswap/src/main.nr` contract we created [previously](./l2_contract_setup.md) in `aztec-contracts/uniswap`, paste these functions:

#include_code swap_private yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust
#include_code assert_token_is_same yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust

This uses a util function `compute_swap_private_content_hash()` - let's add that.

In `util.nr`, add:
#include_code compute_swap_private_content_hash yarn-project/noir-contracts/contracts/uniswap_contract/src/util.nr rust

This flow works similarly to the public flow with a few notable changes:

- Notice how in the `swap_private()`, user has to pass in `token` address which they didn't in the public flow? Since `swap_private()` is a private method, it can't read what token is publicly stored on the token bridge, so instead the user passes a token address, and `_assert_token_is_same()` checks that this user provided address is same as the one in storage. Note that because public functions are executed by the sequencer while private methods are executed locally, all public calls are always done after all private calls are done. So first the burn would happen and only later the sequencer asserts that the token is same. Note that the sequencer just sees a request to `execute_assert_token_is_same` and therefore has no context on what the appropriate private method was. If the assertion fails, then the kernel circuit will fail to create a proof and hence the transaction will be dropped.
- In the public flow, the user calls `transfer_public()`. Here instead, the user calls `unshield()`. Why? The user can't directly transfer their private tokens, their notes to the uniswap contract, because later the Uniswap contract has to approve the bridge to burn these notes and withdraw to L1. The authwit flow for the private domain requires a signature from the `sender`, which in this case would be the Uniswap contract. For the contract to sign, it would need a private key associated to it. But who would operate this key?
- To work around this, the user can unshield their private tokens into Uniswap L2 contract. Unshielding would convert user's private notes to public balance. It is a private method on the token contract that reduces a user’s private balance and then calls a public method to increase the recipient’s (ie Uniswap) public balance. **Remember that first all private methods are executed and then later all public methods will be - so the Uniswap contract won’t have the funds until public execution begins.**
- Now uniswap has public balance (like with the public flow). Hence, `swap_private()` calls the internal public method which approves the input token bridge to burn Uniswap’s tokens and calls `exit_to_l1_public` to create an L2 → L1 message to exit to L1.
- Constructing the message content for swapping works exactly as the public flow except instead of specifying who would be the Aztec address that receives the swapped funds, we specify a secret hash (`secret_hash_for_redeeming_minted_notes`). Only those who know the preimage to the secret can later redeem the minted notes to themselves.

In the next step we will write the code to execute this swap on L1.
