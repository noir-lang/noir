---
title: Recover Accounts
---

Recover an Aztec account.

Aztec allows for recovery of an account for which all of the registered spending keys have been lost. The recovering party will still need the account privacy key to decrypt the associated account notes.

At a high level, what the [RecoverAccountController](./../types/sdk/RecoverAccountController) does is it adds a trusted third party key as a new spending key for the account. This trusted third party key can then be used to [add new spending keys](add-spending-keys) to the account, so only add a recovery account that is managed by someone that you trust!

The `RecoverAccountController` does not require an Aztec account and can be used with only a funded Ethereum account. This allows anyone to initiate an account recovery.

The trusted third party key is **different** than the `recoveryPublicKey` passed to the `RegistrationController` during [setup](register#setup).

## Recovery Payload

In practice, a user generates a [RecoveryPayload](../types/sdk/RecoveryPayload) from their account public key, their alias and a trusted third party public key. The `RecoveryPayload` includes the `recoveryPublicKey`.

To be able to recover an account, a user must have the `RecoveryPayload` generated using the SDK. Generating a `RecoveryPayload` is **not** deterministic, you will not be able to regenerate a `RecoveryPayload` with the same inputs later. Store the recovery payload someplace safe!

Using the SDK to generate a `RecoveryPayload`:

```ts
// Store this third party key! Its what is used to recover the account.
const thirdPartySigner = await sdk.createSchnorrSigner(Buffer.alloc(32, 3, "hex";
const trustedThirdPartyPublicKey = thirdPartySigner.getPublicKey();

// Store this recovery payload! It cannot be regenerated--it includes randomness
const recoveryPayloads = await sdk.generateAccountRecoveryData(
    accounts[defaultAccountIndex].privacyAccount.id,
    alias,
    [trustedThirdPartyPublicKey]
);
```

You can add additional `recoveryPublicKeys` to an account that is already registered using the [AddSpendingKeyController](../types/sdk/AddSpendingKeyController). See [this page](../usage/add-spending-keys) for more info.

## Setup `RecoverAccountController`

```ts
AztecSdk.createRecoverAccountController(
    recoveryPayload: RecoveryPayload, 
    deposit: AssetValue, 
    fee: AssetValue, 
    depositor: EthAddress, 
    provider?: EthereumProvider): 
        Promise<RecoverAccountController>;
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| recoveryPayload | [RecoveryPayload](../types/sdk/RecoveryPayload) | Contains data used to add a trusted third party spending key to an Aztec account. |
| deposit | [AssetValue](./../types/barretenberg/AssetValue) | The `assetId` (number) and `value` (bigint) to deposit. This can be 0. |
| fee | [AssetValue](./../types/barretenberg/AssetValue) | The Aztec network fee for processing the transaction. |
| depositor | [EthAddress](../types/barretenberg/EthAddress) | The Ethereum account from which to deposit funds. |
| provider? | [EthereumProvider](./../types/barretenberg/EthereumProvider) | Optional Ethereum provider.  |

### Returns

| Return Type |  Description |
| --------- | ----------- |
| [RecoverAccountController](../types/sdk/RecoverAccountController) | A user instance with apis bound to the user's account id. |

## Usage

```ts
const controller = await sdk.createRecoverAccountController(
    recoveryPayloads[0],
    deposit,
    tokenTransferFee,
    depositor
);

await controller.createProof();
await controller.depositFundsToContract();
await controller.awaitDepositFundsToContract();
await controller.sign();
let txId = await controller.send();
```