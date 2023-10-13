---
title: Cancelling Deposits
---

A special type of error is an _underpriced transaction_ - it means that a message is inserted on L1, but the attached fee is too low to be included in a rollup block. In such a case your funds could be stuck in the portal and not minted on L2 (lost forever!)

To address this, the Inbox supports cancelling messages after a deadline. However, this must be called by the portal itself, as it will need to "undo" the state changes is made (for example by sending the tokens back to the user).

In your `TokenPortal.sol` smart contract, paste this:

#include_code token_portal_cancel /l1-contracts/test/portals/TokenPortal.sol solidity

To cancel a message, the portal must reconstruct it - this way we avoid storing messages in the portal itself. Note that just as with deposits we need to support cancelling messages for minting privately and publicly.

Note that the portal uses `msg.sender` as the canceller when computing the secret hash. This is an access control mechanism to restrict only the intended address to cancel a message.

Once the message is cancelled on the inbox, we return the funds back to the user.

The inbox requires each message to provide a deadline by which a message must be consumed. After this time, if the message is still not consumed, the message can be cancelled.

In the next step we will write L1 and L2 logic to withdraw funds from L2 to L1.
