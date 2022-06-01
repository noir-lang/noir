---
title: Transfer Notes
---

Transfer asset notes on the Aztec network.

Use the `TransferController` to spend notes within the Aztec rollup.

You can find the type definition of the `TransferController` class [here](../types/TransferController).

### Controller Setup

```ts
AztecSdk.createTransferController(
    userId: AccountId, 
    userSigner: Signer, 
    value: AssetValue, 
    fee: AssetValue, 
    to: AccountId)
        : Promise<TransferController>;
```

### Get Asset Id

The SDK includes a utility function to get the the AssetId (number) from the Ethereum token address. When sending Ether, you can specify the `0x0` address as `EthAddress.ZERO`.

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

### Create Proof & Send

Once the `TransferController` is created, you can create a proof and send the transaction with:

```ts
await tokenTransferController.createProof();
await tokenTransferController.send();
```

You can review the full reference code [here](https://github.com/critesjosh/aztec-sdk-starter/blob/b4611c001133e2ef35180a2953e5651354315834/src/index.ts#L194).
