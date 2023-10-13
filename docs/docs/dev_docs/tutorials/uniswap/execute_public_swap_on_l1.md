---
title: Solidity Code to Execute Swap on L1
---

To execute the swaps on L1, go back to the `UniswapPortal.sol` we [created earlier](./l1_portal.md) in `l1-contracts`.

Under the struct, paste this code that will manage the public flow:

#include_code solidity_uniswap_swap_public l1-contracts/test/portals/UniswapPortal.sol solidity

**What’s happening here?**

1. It fetches the input and output tokens we are swapping. The Uniswap portal only needs to know the portal addresses of the input and output as they store the underlying ERC20 token address.
2. Consumes the `withdraw` message to get input tokens on L1 to itself. This is needed to execute the swap.

   Before it actually can swap, it checks if the provided swap parameters were what the user actually wanted by creating a message content hash (similar to what we did in the L2 contract) to ensure the right parameters are used.

3. Executes the swap and receives the output funds to itself.

   The deadline by which the funds should be swapped is `block.timestamp` i.e. this block itself. This makes things atomic on the L1 side.

4. The portal must deposit the output funds back to L2 using the output token’s portal. For this we first approve the token portal to move Uniswap funds, and then call the portal’s `depositToAztecPublic()` method to transfer funds to the portal and create a L1 → l2 message to mint the right amount of output tokens on L2.

   To incentivize the sequencer to pick up this message, we pass a fee to the deposit message.

This concludes the public flow.

In the next step, we will code a private flow in the Aztec.nr contract.
