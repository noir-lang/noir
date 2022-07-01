---
title: Transfer Assets
---

Transfer asset notes on the Aztec network.

Use the `TransferController` to spend notes (assets) within the Aztec rollup.

You can find the type definition of the `TransferController` class [here](../types/sdk/TransferController).

### Controller Setup

```ts
AztecSdk.createTransferController(
    userId: GrumpkinAddress, 
    userSigner: Signer, 
    value: AssetValue, 
    fee: AssetValue, 
    recipient: GrumpkinAddress, 
    recipientSpendingKeyRequired?: boolean)
        : Promise<TransferController>;
```

### Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | Current owner of the asset note (the sender). |
| userSigner | [Signer](../types/sdk/Signer) | A signer for the sending account. |
| value | [AssetValue](../types/barretenberg/AssetValue) | Asset type and amount to send. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | Asset type and amount to pay for the Aztec transaction fee. |
| recipient | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | Public key of the receiving account. |
| recipientSpendingKeyRequired? | boolean | Optional flag to ensure that the recipient has registered an account. Defaults to true. |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [TransferController](../types/sdk/TransferController) | A user instance with apis bound to the user's account id. |

### Get Asset Id

The SDK includes a utility function to get the the AssetId (assetId: number) from the Ethereum token address. When sending Ether, you can specify the `0x0` address as `EthAddress.ZERO`.

```ts
const assetId = sdk.getAssetIdByAddress(tokenAddress);
const zkETH = sdk.getAssetIdByAddress(EthAddress.ZERO);
```

### Transfer fees

You calculate transfer fees similar to how it is done with registrations or deposits.

```ts
const tokenTransferFee = (await sdk.getTransferFees(assetId))[settlementTime];
```

Where `settlementTime` is `TxSettlementTime.INSTANT` or `TxSettlementTime.NEXT_ROLLUP`. `INSTANT` settlement is faster, but more expensive. `NEXT_ROLLUP` will wait until the rollup is filled with transactions and then is posted to Ethereum.

The settlement time is inferred from the fee a user pays, it is not explicitly sent to the controller.

### Create Proof & Send

Once the `TransferController` is created, you can create a proof and send the transaction with:

```ts
await tokenTransferController.createProof();
await tokenTransferController.send();
```

You can review the full reference code [here](https://github.com/critesjosh/aztec-sdk-starter/blob/mainnet-fork/src/latest/transferNotes.ts).
