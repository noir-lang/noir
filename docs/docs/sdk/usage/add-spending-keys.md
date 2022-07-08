---
title: Add Spending Keys
---

Add spending keys to a registered Aztec account.

The [Accounts page](../../how-aztec-works/accounts) contains helpful context if you are unfamiliar with how Aztec accounts work.

There is no limit to the number of spending keys that can be added to an Aztec account. This decreases the need for insecure key sharing between devices. For example, you can register a unique spending key on each one of your devices for the same Aztec account, so you don't have to copy and paste private keys.

You can also use the [AddSpendingKeyController](../types/sdk/AddSpendingKeyController) to add additional recovery public keys to an account after it has been registered. Read more about account recovery [here](account-recovery).

## Setup

```ts
AztecSdk.createAddSpendingKeyController(
    userId: GrumpkinAddress, 
    userSigner: Signer, 
    spendingPublicKey1: GrumpkinAddress, 
    spendingPublicKey2: GrumpkinAddress | undefined, 
    fee: AssetValue): 
        AddSpendingKeyController;
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key of the account registering the new signing keys. |
| userSigner | [Signer](../types/sdk/Signer) | A signer associated with the userId. |
| spendingPublicKey1 | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key of a new spending key. |
| spendingPublicKey2 | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key of a new spending key. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | The Aztec transaction fee. |

### Returns

| Return Type | Description |
| --- | --- |
| [AddSpendingKeyController](../types/sdk/AddSpendingKeyController) | A user instance with apis bound to the user's account id. |

## Usage

The follow code is an example of how you could set up and use the `AddSpendingKeyController`. Obviously, you'll want to save the the spending key private keys to use them later or generate them using a different method.

```ts
const newSpendingKey1 = await sdk.createSchnorrSigner(randomBytes(32));
const newSpendingKey2 = await sdk.createSchnorrSigner(randomBytes(32));

const fee = (await sdk.getAddSpendingKeyFees(
    sdk.getAssetIdBySymbol("ETH")))[TxSettlementTime.NEXT_ROLLUP];

const controller = sdk.createAddSpendingKeyController(
    user,
    signer,
    newSpendingKey1.getPublicKey(),
    newSpendingKey2.getPublicKey(),
    fee
);
await controller.createProof();
let txId = await controller.send();
```

### Transaction Fees

The transaction fee can be paid in ETH or DAI.

The settlement time can either be `NEXT_ROLLUP` or `INSTANT`. Refer to the [fees section](./register#calculating-fees) on the registration page for a more detailed explanation.
