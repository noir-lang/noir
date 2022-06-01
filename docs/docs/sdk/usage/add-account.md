---
title: Add Accounts to the SDK
---

Add accounts with the privacy key.

Because all notes representing value on Aztec network are encrypted, the SDK requires access to a user's privacy key in order to decrypt the notes and calculate a user's balance.

Please review [the page on Accounts](../../how-aztec-works/accounts.md) if you haven't already as it will help you understand the difference between Ethereum and Aztec accounts.

:::info

Aztec will support the same elliptic curve as Ethereum in the future so Ethereum accounts will be Aztec accounts.

:::

## Privacy Keys

Privacy keys can be any random 32 bytes. Zk.money generates keys deterministically from Ethereum accounts by deriving the keys from signed messages.

You can generate the privacy key from an Ethereum key by signing this message:

`Sign this message to generate your Aztec Privacy Key. This key lets the application decrypt your balance on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`

and taking the first 32 bytes of the resulting signed message. You can find an example [here](https://github.com/critesjosh/aztec-sdk-starter/blob/3abc0b24b0570198a7c5492f7de8d7f452c910fa/src/aztecKeys.ts#L21).

With the privacy key, adding the user to the SDK is as simple as passing the key and the nonce for the account.

```ts
const account0 = await AztecSdk.addUser(privacyKey, 0);
const account1 = await AztecSdk.addUser(privacyKey, 1);
```

Now just make sure the SDK has synced and you can read account balances:

```ts
await account1.awaitSynchronised();
const ethBalance = await account1.getBalance(AztecSdk.getAssetIdBySymbol("ETH"))
```

## Spending Keys

A spending key is required to spend notes on the network. By default, the spending key is the same as the privacy key at nonce 0, so it is considered best practice to register a new account (same public key, but with nonce 1) with a new spending key. You can read more about registering new accounts for users on the [Register Users page](./register-user). Here, we will briefly review how to add an account that has already been registered to the SDK.

You can create an Aztec signer by passing the signing key to the `createSchnorrSigner(privateKey: Buffer)` method. It returns a signer that you can pass to various `Controllers` to sign transactions on the network on behalf of the account.

```ts
const signer1 = await AztecSdk.createSchnorrSigner(signingPrivateKey);
```

Like privacy keys, spending keys can be 32 random bytes, but zk.money generates them deterministically from Ethereum accounts by deriving them from a signed message. The message used for creating signing keys is:

`Sign this message to generate your Aztec Spending Key. This key lets the application spend your funds on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`

You can see an example in the reference repo [here](https://github.com/critesjosh/aztec-sdk-starter/blob/b4611c001133e2ef35180a2953e5651354315834/src/index.ts#L89).

## Add User

```ts
AztecSdk.addUser(privateKey: Buffer, accountNonce?: number, noSync?: boolean): Promise<AztecSdkUser>
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| privateKey | Buffer | The privacy key of the user. |
| nonce | number (optional) | The nonce of the user. Default to the latest nonce. |
| noSync | boolean | Whether to skip sync. Default is `false`.  |

| Return Type | Description |
| --------- | ----------- |
| [AztecSdkUser](../types/AztecSdkUser.md) | A user instance with apis bound to the user's account id. |

## Get User

```ts
AztecSdk.getUserData(userId: AccountId): UserData
// or
user.getUserData();
```

| Return Type | Description |
| --------- | ----------- |
| [UserData](./../types/UserData.md) | Info about the current user. |