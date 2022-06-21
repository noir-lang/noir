---
title: Shield Assets (Deposit)
---

Deposit Assets from Ethereum to Aztec.

The SDK comes with a `DepositController` that makes it easy to create and track deposit transactions from Ethereum to Aztec.

Deposits require sending Ethereum transactions from a user's account to the Aztec deposit contract.

You can find the interface for the `DepositController` class [here](../types/sdk/DefiController).

### Controller Setup

```ts
AztecSdk.createDepositController(
    depositor: EthAddress,          
    value: AssetValue,               
    fee: AssetValue,                  
    recipient: GrumpkinAddress,  
    recipientSpendingKeyRequired?: boolean,
    feePayer?: FeePayer,
    provider?: EthereumProvider)
        : Promise<DepositController>
```

### Controller Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| depositor | [EthAddress](../types/barretenberg/EthAddress) | Ethereum account making the deposit. |
| value | [AssetValue](../types/barretenberg/AssetValue) | Type and amount of deposit. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | Type and amount for the Aztec transaction fee. |
| recipient | [GrupmkinAddress](../types/barretenberg/GrumpkinAddress) | The account public key of the Aztec account. |
| recipientSpendingKeyRequired? | boolean | Optional flag that specifies whether the recipient account should already be registered. Defaults to `true`.|
| feePayer | [FeePayer](../types/sdk/FeePayer) | Optional input that can specify an alternative Aztec account to pay tx fee. |
| provider | [EthereumProvider](../types/barretenberg/EthereumProvider) | Ethereum provider. |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [DepositController](../types/sdk/DepositController) | A user instance with apis bound to the user's account id. |
### Executing a Deposit

The complete deposit flow for Ether using the `DepositController` looks like this: 

```ts
const tokenAssetId = sdk.getAssetIdBySymbol('ETH');
const tokenDepositFee = (await sdk.getDepositFees(tokenAssetId))[
    settlementTime
];
const tokenAssetValue: AssetValue = {
    assetId: tokenAssetId,
    value: tokenQuantity,
};
const tokenDepositController = sdk.createDepositController(
    depositor,
    tokenAssetValue,
    tokenDepositFee,
    recipient,
    true,
);
await tokenDepositController.createProof();
await tokenDepositController.sign();
// check if there are pending depsoits
if ((await tokenDepositController.getPendingFunds()) < tokenQuantity) {
    await tokenDepositController.depositFundsToContract();
    await tokenDepositController.awaitDepositFundsToContract();
}
let txId = await tokenDepositController.send();
```

You can review the complete reference script [here](https://github.com/critesjosh/aztec-sdk-starter/blob/mainnet-fork/src/latest/shieldAssets.ts).

#### Required Approvals

When depositing an ERC-20 token like DAI, you will need to approve Aztec as an authorized spender before depositing. The `DepositController` includes a method for this, `DepositController.approve()` which will request approval for the amount required for the deposit.
