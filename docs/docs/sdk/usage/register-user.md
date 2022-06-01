---
title: Register Accounts
---

Please review the [Accounts overview](../../how-aztec-works/accounts) and [Add User](./add-user) pages if you haven't already.

As mentioned in the above pages, there is an important difference between the account nonce identifiers 0 and 1 in the Aztec account system. The privacy key associated with nonce 0 is used to decrypt notes, calculate balances and track transactions associated with an account. The signing keys associated with account nonce 1 are used to spend notes on behalf of a user.

Account with nonce 1 must be registered on the network before it can be used. The SDK makes it easy to register a new account using the `RegisterController`.

## RegisterController Setup

You can find the type definition of the RegisterController class [here](../types/RegisterController).

`AztecSdk.createRegisterController`

```ts
AztecSdk.createRegisterController(
    userId: AccountId,     // privacy account (nonce 0)
    userSigner: Signer,    // account 0 signer
    alias: string,         // arbitrary account identifier
    signingPublicKey: GrumpkinAddress,  // signing key
    recoveryPublicKey: GrumpkinAddress | undefined, // optional recovery key
    deposit: AssetValue,   // deposit asset & amount
    fee: AssetValue,       // network fee
    depositor: EthAddress, // depositers Ethereum account 
    provider?: EthereumProvider)
        : Promise<RegisterController>;
```

### Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [AccountId](../types/AccountId) | The AccountId of the account with nonce 0 registering the new account. |
| userSigner | [Signer](../types/Signer) | The signer for AccountId with nonce 0. |
| alias | string | The alias to register the new account with. This is typically a human-readable, easy to remember identifier. |
| signingPublicKey | [GrumpkinAddress](../types/GrumpkinAddress) | The public key for the new account. Users must remember the corresponding private key (or the derivation method). |
| recoveryPublicKey | [GrumpkinAddress](../types/GrumpkinAddress) | An optional recovery key that allows the account to be recovered if the `signingPublicKey` is lost. |
| deposit | [AssetValue](../types/AssetValue) | The `assetId` (number) and `value` (bigint) to deposit. |
| fee | [AssetValue](../types/AssetValue) | The network fee for registering the account. |
| depositor | [EthAddress](../types/EthAddress) | The Ethereum account from which to deposit the funds. |
| provider | [EthereumProvider](../types/EthereumProvider) | Optional Ethereum Provider. |

### Calculating Fees

`(await AztecSdk.getRegisterFees(deposit: AssetValue))[settlementTime: TxSettlementTime]` will help determine how much the transaction will cost. Settlement time can either be `NEXT_ROLLUP` or `INSTANT`. `INSTANT` is more expensive because it indicates you will pay for the rollup to settle "instantly" (on the order of minutes rather than hours) rather than waiting for it to fill up with other transactions. When there are more transactions in the rollup, users are splitting the cost among more transactions so each one is cheaper.

### Full Registration Flow

Once the `RegisterController` has been created, you can use it to deposit funds, create proofs, and sign and send transactions. For example:

```ts
const assetId = sdk.getAssetIdByAddress(tokenAddress);
const deposit = { assetId, value: tokenQuantity };
const txFee = (await sdk.getRegisterFees(deposit))[TxSettlementTime.NEXT_ROLLUP];

const controller = await sdk.createRegisterController(
    user,
    signer,
    alias,
    newSigner.getPublicKey(),
    recoveryPublicKey,
    deposit,
    txFee,
    depositer
);

await controller.depositFundsToContract();
await controller.awaitDepositFundsToContract();

await controller.createProof();
await controller.sign();
await controller.send();
```

You can find the full example script that calls this method [here](https://github.com/critesjosh/aztec-sdk-starter/blob/main/src/registerAccount.ts).
