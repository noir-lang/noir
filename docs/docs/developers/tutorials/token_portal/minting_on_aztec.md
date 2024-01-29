---
title: Minting tokens on Aztec
---

In this step we will start writing our Aztec.nr bridge smart contract and write a function to consume the message from the token portal to mint funds on Aztec

## Initial contract setup

In our `token-bridge` Aztec project in `aztec-contracts`, under `src` there is an example `main.nr` file. Paste this to define imports and initialize the constructor:

#include_code token_bridge_imports /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

#include_code token_bridge_storage_and_constructor /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

This imports Aztec-related dependencies and our helper file `token_interface.nr`.
(The code above will give errors right now - this is because we haven't implemented util and token_interface yet.)

In `token_interface.nr`, add this:

#include_code token_bridge_token_interface /yarn-project/noir-contracts/contracts/token_bridge_contract/src/token_interface.nr rust

## Consume the L1 message

In the previous step, we have moved our funds to the portal and created a L1->L2 message. Upon building the next rollup, the sequencer asks the inbox for any incoming messages and adds them to Aztec’s L1->L2 message tree, so an application on L2 can prove that the message exists and consumes it.

In `main.nr`, now paste this `claim_public` function:
#include_code claim_public /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

The `claim_public` function enables anyone to consume the message on the user's behalf and mint tokens for them on L2. This is fine as the minting of tokens is done publicly anyway.

**What’s happening here?**

1. We first recompute the L1->L2 message content by calling `get_mint_public_content_hash()`. Note that the method does exactly the same as what the TokenPortal contract does in `depositToAztecPublic()` to create the content hash.
2. We then attempt to consume the L1->L2 message by passing the `msg_key`, the the content hash, and the "secret". Since we are depositing to Aztec publicly, this secret is public, anyone can know this and is usually 0.
   - `context.consume_l1_to_l2_message()` takes in the content_hash and secret to recreate the original message. The L1 to L2 message consists of:
     - Sender - who on L1 sent the message + chain ID of L1. The context variable knows the portal address on L1 and adds that
     - Recipient - i.e. this aztec contract address which is consuming the message + the current version of the aztec rollup.
     - The content - which is reconstructed in the `get_mint_public_content_hash()`
   - Note that the `content_hash` requires `to`, `amount` and `canceller`. If a malicious user tries to mint tokens to their address by changing the to address, the content hash will be different to what the token portal had calculated on L1 and the `msg_Key` will also be different, thus preventing the L1->L2 message from being consumed. This is why we add these parameters into the content.
3. Then we call `Token::at(storage.token.read()).mint_public()` to mint the tokens to the to address.

## Private flow

Now we will create a function to mint the amount privately. Paste this into your `main.nr`

#include_code claim_private /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

#include_code call_mint_on_token /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

The `get_mint_private_content_hash` function is imported from the `token_portal_content_hash_lib`.

If the content hashes were constructed similarly for `mint_private` and `mint_publicly`, then content intended for private execution could have been consumed by calling the `claim_public` method. By making these two content hashes distinct, we prevent this scenario.

While we mint the tokens on L2, we _still don’t actually mint them to a certain address_. Instead we continue to pass the `secret_hash_for_redeeming_minted_notes` like we did on L1. This means that a user could reveal their secret for L2 message consumption for anyone to mint tokens on L2 but they can redeem these notes at a later time. **This enables a paradigm where an app can manage user’s secrets for L2 message consumption on their behalf**. **The app or any external party can also mint tokens on the user’s behalf should they be comfortable with leaking the secret for L2 Message consumption.** This doesn’t leak any new information to the app because their smart contract on L1 knew that a user wanted to move some amount of tokens to L2. The app still doesn’t know which address on L2 the user wants these notes to be in, but they can mint tokens nevertheless on their behalf.

To mint tokens privately, `claim_private` calls an internal function `_call_mint_on_token()` which then calls [token.mint_private()](../writing_token_contract.md#mint_private) which is a public method since it operates on public storage. Note that mint_private (on the token contract) is public because it too reads from public storage. Since the `secret_hash_for_redeeming_minted_notes` is passed publicly (and not the secret), nothing that should be leaked is, and the only the person that knows the secret can actually redeem their notes at a later time by calling [`Token.redeem_shield(secret, amount)`](../writing_token_contract.md#redeem_shield).

In the next step we will see how we can cancel a message.
