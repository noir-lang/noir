---
title: Withdraw Assets
---

Withdraw assets from Aztec to Ethereum.

Use the `WithdrawController` to withdraw assets from Aztec to Ethereum.

You can find the interface of the `WithdrawController` [here](../types/WithdrawController).

```ts
AztecSdk.createWithdrawController(
    userId: AccountId, 
    userSigner: Signer, 
    value: AssetValue, 
    fee: AssetValue, 
    to: EthAddress)
        : Promise<WithdrawController>
```

### Fees

Fees for withdrawals are calculated using a similar method as for [registrations](register-user#calculating-fees), deposits and [transfers](transfer#transfer-fees), but using the `getWithdrawalFees` method.
### Executing a Withdrawal

A withdrawal setup and execution looks like this:

```ts
const tokenAssetId = sdk.getAssetIdByAddress(tokenAddress);
const tokenWithdrawFee = (await sdk.getWithdrawFees(tokenAssetId))[settlementTime];

const tokenAssetValue = { assetId: tokenAssetId, value: tokenQuantity };
const tokenWithdrawController = sdk.createWithdrawController(
    user,
    signer,
    tokenAssetValue,
    tokenWithdrawFee,
    recipientEthereumAddress
);

await tokenWithdrawController.createProof();
await tokenWithdrawController.send();
```

Once the transaction is sent, you just have to wait for the rollup to settle on Ethereum.