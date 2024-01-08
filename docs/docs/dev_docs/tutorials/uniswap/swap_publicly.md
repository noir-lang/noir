---
title: Swapping Publicly
---

In this step we will create the flow for allowing a user to swap their tokens publicly on L1. It will have the functionality of letting anyone call this method on behalf of the user, assuming they have appropriate approvals. This means that an operator can pay gas fees on behalf of the user!

In `main.nr` paste this:

#include_code swap_public yarn-project/noir-contracts/contracts/uniswap_contract/src/main.nr rust

This uses a util function `compute_swap_public_content_hash()` - let's add that.

In `util.nr`, add:
#include_code uniswap_public_content_hash yarn-project/noir-contracts/contracts/uniswap_contract/src/util.nr rust

**What’s happening here?**

1. We check that `msg.sender()` has appropriate approval to call this on behalf of the sender by constructing an authwit message and checking if `from` has given the approval (read more about authwit [here](../../contracts/resources/common_patterns/authwit.md)).
2. We fetch the underlying aztec token that needs to be swapped.
3. We transfer the user’s funds to the Uniswap contract. Like with Ethereum, the user must have provided approval to the Uniswap contract to do so. The user must provide the nonce they used in the approval for transfer, so that Uniswap can send it to the token contract, to prove it has appropriate approval.
4. Funds are added to the Uniswap contract.
5. Uniswap must exit the input tokens to L1. For this it has to approve the bridge to burn its tokens on its behalf and then actually exit the funds. We call the [`exit_to_l1_public()` method on the token bridge](../token_portal/withdrawing_to_l1.md). We use the public flow for exiting since we are operating on public state.
6. It is not enough for us to simply emit a message to withdraw the funds. We also need to emit a message to display our swap intention. If we do not do this, there is nothing stopping a third party from calling the Uniswap portal with their own parameters and consuming our message.

So the Uniswap portal (on L1) needs to know:

- The token portals for the input and output token (to withdraw the input token to L1 and later deposit the output token to L2)
- The amount of input tokens they want to swap
- The Uniswap fee tier they want to use
- The minimum output amount they can accept (for slippage protection)

The Uniswap portal must first withdraw the input tokens, then check that the swap message exists in the outbox, execute the swap, and then call the output token to deposit the swapped tokens to L2. So the Uniswap portal must also be pass any parameters needed to complete the deposit of swapped tokens to L2. From the tutorial on building token bridges we know these are:

- The address on L2 which must receive the output tokens (remember this is public flow)
- The secret hash for consume the L1 to L2 message. Since this is the public flow the preimage doesn’t need to be a secret
- The deadline to consume the l1 to l2 message (this is so funds aren’t stuck in the processing state forever and the message can be cancelled. Else the swapped tokens would be stuck forever)
- The address that can cancel the message (and receive the swapped tokens)

6. We include these params in the L2 → L1 `swap_public message content` too. Under the hood, the protocol adds the sender (the Uniswap l2 contract) and the recipient (the Uniswap portal contract on L1).

In the next step we will write the code to execute this swap on L1.
