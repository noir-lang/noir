---
title: Withdraw Assets
---

Withdraw assets from Aztec to Ethereum.

Use the `WithdrawController` to withdraw assets from Aztec to Ethereum.

You can find the interface of the `WithdrawController` [here](../types/sdk/WithdrawController).

### Controller Setup

```ts
AztecSdk.createWithdrawController(
    userId: GrumpkinAddress, 
    userSigner: Signer, 
    value: AssetValue, 
    fee: AssetValue, 
    to: EthAddress)
        : Promise<WithdrawController>
```

### Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The Aztec account to make the withrawal from. |
| userSigner | [Signer](../types/sdk/Signer) | A signer for the provided `userId`. |
| value | [AssetValue](../types/barretenberg/AssetValue) | The asset type and amount to withdraw. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | The asset type and amount to pay for the Aztec transaction fee. |
| to | [EthAddress](../types/barretenberg/EthAddress) | The Ethereum address to send the funds on Ethereum. |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [WithdrawController](../types/sdk/WithdrawController) | A user instance with apis bound to the user's account id. |

### Fees

Fees for withdrawals are calculated using a similar method as for [registrations](register-#calculating-fees), deposits and [transfers](transfer#transfer-fees), but using the `getWithdrawalFees` method.

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
let txId = await tokenWithdrawController.send();
```

Once the transaction is sent, you just have to wait for the rollup to settle on Ethereum and the Rollup processor contract will send the funds.
