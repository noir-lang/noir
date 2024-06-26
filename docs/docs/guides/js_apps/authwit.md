---
title: How to use authentication witnesses (authwit)
tags: [accounts]
---

This page assumes you have authwit set up correctly in your contract. To learn how to do that, [go here](../smart_contracts/writing_contracts/authwit.md).

For an introduction to authentication witnesses on Aztec, [read this explainer](../../aztec/concepts/accounts/authwit.md).

## Import libraries

These are all the libraries you might need for using authwits in Aztec.js:

```typescript
import {
  computeAuthWitMessageHash,
  computeInnerAuthWitHash,
} from "@aztec/aztec.js";
```

You may not need all of these.

## Publicly deploy accounts

:::note
This is only required if you are using authwits in public
:::

If you are using public authwit (ie using `assert_current_call_valid_authwit_public` in your contract), you will need to deploy the following accounts publicly:

1. The account that is giving permission to an account to act on behalf of it (authwit giver)
2. The account that does the action (authwit receiver)

Here is an example implementation:

#include_code public_deploy_accounts yarn-project/end-to-end/src/fixtures/utils.ts typescript

You would then call this like so:

#include_code public_deploy_accounts yarn-project/end-to-end/src/e2e_authwit.test.ts typescript

## Define the action

When creating an authwit, you will need to pass the authwit giver, the authwit receiver (who will perform the action), and the action that is being authorized. The action can be a smart contract function call, or alternatively, arbitrary data.

### When the action is a function call

You can define the action like this:

#include_code authwit_computeAuthWitMessageHash yarn-project/end-to-end/src/e2e_blacklist_token_contract/transfer_private.test.ts typescript

In this example,

- `asset` refers to a token contract
- `withWallet(wallets[1])` is specifying the authwit receiver (`wallets[1]`) will do this action
- `.methods.transfer()` is specifying that the action is calling the `transfer` method on the token contract
- `(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);` are the args of this method - it will send the `amount` from `wallets[0]` to `wallets[1]`

### Arbitrary message

You can hash your own authwit message by creating an inner hash with the data, like this:

#include_code compute_inner_authwit_hash yarn-project/end-to-end/src/e2e_authwit.test.ts typescript

Then create the message hash by hashing the inner hash with the authwit receiver address, chainId, and version:

#include_code compute_arbitrary_authwit_hash yarn-project/end-to-end/src/e2e_authwit.test.ts typescript

## Create the authwit

There are slightly different interfaces depending on whether your contract is checking the authwit in private or public. 

Public authwits are stored in the account contract and batched with the authwit action call, so a user must send a transaction to update their account contract, authorizing an action before the authorized contract's public call will succeed. 

Private execution uses oracles and are executed locally by the PXE, so the authwit needs to be created by the authwit giver and then added to the authwit receiver's PXE.

### Private

This is expected to be used alongside [private authwits in Aztec.nr contract](../smart_contracts/writing_contracts/authwit.md#private-functions).

Create a private authwit like this:

#include_code create_authwit yarn-project/end-to-end/src/e2e_blacklist_token_contract/transfer_private.test.ts typescript

In this example,

- `wallets[0]` is the authwit giver
- `wallets[1]` is the authwit reciever and caller of the function
- `action` was [defined previously](#define-the-action)

If you created an arbitrary message, you can create the authwit by replacing these params with the outer hash:

#include_code compute_arbitrary_authwit_hash yarn-project/end-to-end/src/e2e_authwit.test.ts typescript

Then add it to the wallet of the authwit receiver (the caller of the function):

#include_code add_authwit yarn-project/end-to-end/src/e2e_blacklist_token_contract/transfer_private.test.ts typescript

### Public

This is expected to be used alongside [public authwits in Aztec.nr contract](../smart_contracts/writing_contracts/authwit.md#public-functions).

Set a public authwit like this:

#include_code set_public_authwit yarn-project/end-to-end/src/e2e_blacklist_token_contract/transfer_public.test.ts typescript

Remember it is a transaction and calls a method in the account contract. In this example,

- `wallets[0]` is the authwit giver
- `wallets[1]` is the authwit reciever and caller of the function
- `action` was [defined previously](#define-the-action)
- `true` sets the `authorized` boolean (`false` would revoke this authwit)

If you created an arbitrary message, you would replace the first param struct with the outer hash:

#include_code set_public_authwit yarn-project/end-to-end/src/e2e_authwit.test.ts typescript

## Further reading

- [An explainer of authentication witnesses](../../aztec/concepts/accounts/authwit.md)
- [Authwits in Aztec.nr](../smart_contracts/writing_contracts/authwit.md)
