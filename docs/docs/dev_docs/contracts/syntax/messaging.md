---
title: Messaging
description: Documentation of Aztec's Messaging system
---

# Messaging

## L1 --> L2
The context available within functions includes the ability to send messages to l1. For more information on how cross chain communication works in Aztec, see the [documentation on communication.](../../../concepts/foundation/communication/cross_chain_calls.md)

#include_code non_native_token_withdraw  /yarn-project/noir-contracts/src/contracts/non_native_token_contract/src/main.nr rust

### What happens behind the scenes?
When a user sends a message from a [portal contract](../../../concepts/foundation/communication/cross_chain_calls.md#portal) to the rollup's inbox it gets processed and added to the `l1 to l2 messages tree`.

 <-- TODO(Maddiaa): INCLUDE LINK TO WHERE the messages tree is discussed elsewhere in the docs. -->

The l1 to l2 messages tree contains all messages that have been sent from l1 to the l2. The good thing about this tree is that it does not reveal when it's messages have been spent, as consuming a message from the l1 to l2 messages tree is done by nullifing a message.

When calling the `consume_l1_to_l2_message` function on a contract; a number of actions are performed by `Aztec.nr`.

1. The `msgKey` value (passed to the consume message function) is used to look up the contents of the l1 message.
2. Check that the message recipient is the contract of the current calling context.
3. Check that the message content matches the content reproduced earlier on. 
4. Validate that caller know's the preimage to the message's `secretHash`. See more information [here](../../../concepts/foundation/communication/cross_chain_calls.md#messages).
5. We compute the nullifier for the message.
#include_code l1_to_l2_message_compute_nullifier  /yarn-project/aztec-nr/aztec/src/messaging/l1_to_l2_message.nr rust
6. Finally we push the nullifier to the context. Allowing it to be checked for validity by the kernel and rollup circuits. 

#include_code consume_l1_to_l2_message  /yarn-project/aztec-nr/aztec/src/context.nr rust

As the same nullifier cannot be created twice. We cannot consume the message again.

## L2 ->> L1
