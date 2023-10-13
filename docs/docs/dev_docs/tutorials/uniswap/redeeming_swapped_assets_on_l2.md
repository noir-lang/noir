---
title: Redeeming Swapped Assets on L2
---
So you emitted a message to withdraw input tokens to L1 and a message to swap. Then you or someone on your behalf can swap on L1 and emit a message to deposit swapped assets to L2,

You still need to "claim" these swapped funds on L2.

In the public flow, you can call [`claim_public()`](../token_portal/minting_on_aztec.md) on the output token bridge which consumes the deposit message and mints your assets.

In the private flow, you can choose to leak your secret for L1 → L2 message consumption to let someone mint the notes on L2 (by calling [`claim_private()`](../token_portal/minting_on_aztec.md) on the output token bridge) and then you can later redeem these notes to yourself by presenting the preimage to `secret_hash_for_redeeming_minted_notes` and calling the `redeem_shield()` method on the token contract.

In the next step we will write the typescript code that interacts with all these contracts on the sandbox to actually execute the swaps!