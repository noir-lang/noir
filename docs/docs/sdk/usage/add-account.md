---
title: Add Accounts to the SDK
---

Add accounts with the account (privacy) key.

Because all notes representing value on Aztec network are encrypted, the SDK requires access to a user's account key in order to decrypt the notes and calculate the account balance.

Please review [the page on Accounts](../../how-aztec-works/accounts.md) if you haven't already as it will help you understand the difference between Ethereum and Aztec accounts.

:::info

Aztec will support the same elliptic curve as Ethereum in the future so Ethereum accounts will be Aztec accounts.

:::

## Account Keys

Privacy keys can be any random 32 bytes. The SDK allows you to generate keys deterministically from Ethereum accounts by deriving Aztec keys from signed Ethereum messages.

### Generate

You can generate the account key from an Ethereum private key by signing this message:

`Sign this message to generate your Aztec Privacy Key. This key lets the application decrypt your balance on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`

and taking the first 32 bytes of the resulting signed message. You can find an example in the SDK [here](https://github.com/AztecProtocol/aztec-connect/blob/ec87601503c6425b6a578a19117ead5a582df91c/sdk/src/aztec_sdk/aztec_sdk.ts#L196).

```ts
const { publicKey, privateKey } = sdk.generateAccountKeyPair(ethereumAccount);
```

### Add to SDK

With the account key, adding the user to the SDK is as simple as passing the key and the nonce for the account.

```ts
const account = await sdk.addUser(accountKey);
```

### Read

Now just make sure the SDK account has synced and you can read account balances:

```ts
await account.awaitSynchronised();
const zkEthBalance = await account.getBalance(sdk.getAssetIdBySymbol("ETH"));
```

## Spending Keys

Once a spending key has been registered, either the account key or a registered spending key can be used to spend notes. This creates a useful separation between account (or "viewing/privacy keys") and spending keys. The sender of a note can specify whether the note is spendable by the account key or a registered spending key. This is specified with the `recipientSpendingKeyRequired` flag when setting up a controller. See the [TransferController](./transfer#controller-setup) for an example.

It is considered best practice to register a new spending key as soon as possible. You can read more about registering new accounts for users on the [Register Users page](./register). Here, we will briefly review how to add a spending key to the SDK that has already been registered.

You can create an Aztec signer by passing the signing key to the `createSchnorrSigner(privateKey: Buffer)` method. It returns a signer that you can pass to various `Controllers` to sign transactions on the network on behalf of the account.

```ts
const signer = await sdk.createSchnorrSigner(signingPrivateKey);
```

Like account keys, spending keys can be 32 random bytes, but the SDK allows you to generate them deterministically from Ethereum accounts by deriving them from a signed message. The message used for creating signing keys is:

`Sign this message to generate your Aztec Spending Key. This key lets the application spend your funds on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application.`

You can see an example in the SDK source code [here](https://github.com/AztecProtocol/aztec-connect/blob/ec87601503c6425b6a578a19117ead5a582df91c/sdk/src/aztec_sdk/aztec_sdk.ts#L205).

## Add User

```ts
sdk.addUser(accountPrivateKey: Buffer, noSync?: boolean): Promise<AztecSdkUser>;
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| privateKey | Buffer | The privacy key of the user. |
| noSync | boolean | Whether to skip sync. Default is `false`.  |

| Return Type | Description |
| --------- | ----------- |
| [AztecSdkUser](../types/sdk/AztecSdkUser) | A user instance with apis bound to the user's account id. |

## Get User

```ts
sdk.getUser(userId: GrumpkinAddress): Promise<AztecSdkUser>;
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key of the user. |

| Return Type | Description |
| --------- | ----------- |
| [AztecSdkUser](./../types/sdk/AztecSdkUser) | A user instance with apis bound to the user's account id. |