---
title: Add a User to the SDK
---

Add a user with the privacy key.

Because all notes representing value on Aztec network are encrypted, the SDK requires access to a user's privacy key in order to decrypt the notes and calculate a user's balance.

Please review [the page on Accounts](../../how-aztec-works/accounts.md) if you haven't already as it will help you understand the difference between Ethereum and Aztec accounts.

:::info

Aztec will support the same elliptic curve as Ethereum in the future so Ethereum accounts will be Aztec accounts.

:::

## Privacy Keys

You can generate the privacy key from an Ethereum key by signing this message:

`Sign this message to generate your Aztec Privacy Key. This key lets the application decrypt your balance on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`

and taking the first 32 bytes of the resulting message. This is how it is done in zk.money. You can find an example [here](https://github.com/critesjosh/aztec-sdk-starter/blob/3abc0b24b0570198a7c5492f7de8d7f452c910fa/src/aztecKeys.ts#L21).

With the privacy key, adding the user to the SDK is as simple as passing the key and the nonce for the account.

```ts
const user0 = await sdk.addUser(privateKey, 0);
const user1 = await sdk.addUser(privateKey, 1);
```

Now just make sure the SDK has synced and you can read account balances:

```ts
await user1.awaitSynchronised();
const ethBalance = await user1.getBalance(sdk.getAssetIdBySymbol("ETH"))
```

## Spending Keys

A spending key is required to spend notes on the network. By default, the spending key is the same as the privacy key at nonce 0, so it is considered best practice to register a new account (same public key, but with nonce 1) with a new spending key. You can read more about registering new accounts for users on the [Register Users page](./register-user). Here, we will briefly review how to add an account that has already been registered to the SDK.

You can create an Aztec signer by passing the signing key to the `createSchnorrSigner()` method. It returns a signer that you can pass to various `Controllers` to sign transactions on the network on behalf of the account.

```ts
const signer1 = await sdk.createSchnorrSigner(signingPrivateKey);
```

`Sign this message to generate your Aztec Spending Key. This key lets the application spend your funds on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`
