---
title: Shield Assets (Deposit)
---

The SDK comes with a `DepositController` that makes it easy to create and track deposit transactions from Ethereum to Aztec.

Deposits require sending Ethereum transactions from a user's account to the Aztec deposit contract.

```ts
AztecSdk.createDepositController(
    userId: AccountId,   // Aztec account Id to deposit funds to
    userSigner: Signer,  // signer for the Aztec account
    value: AssetValue,   // amount and type of asset to deposit
    fee: AssetValue,     // Aztec transaction fee
    from: EthAddress,    // Ethereum address sending the funds
    to?: AccountId,      // optionally specify a different recipient
    provider?: EthereumProvider)
        : Promise<DepositController>
```

The complete deposit flow for Ether using the `DepositController` looks like this: 

```ts
const tokenAssetId = sdk.getAssetIdBySymbol('ETH');
const tokenDepositFee = (await sdk.getDepositFees(tokenAssetId))[settlementTime];
const tokenAssetValue: AssetValue = {
    assetId: tokenAssetId,
    value: tokenQuantity,
};

const tokenDepositController = sdk.createDepositController(
    user,
    signer,
    tokenAssetValue,
    tokenDepositFee,
    usersEthereumAddress
);

await tokenDepositController.createProof();
await tokenDepositController.sign();
await tokenDepositController.depositFundsToContractWithProofApproval(); // for ETH, returns txHash
await tokenDepositController.awaitDepositFundsToContract();
await tokenDepositController.send();
```

When depositing an ERC-20 token like DAI, you will need to approve Aztec as an authorized spender before depositing. The `DepositController` includes a method for this, `DepositController.approve()` which will request approval for the amount required for the deposit.

You can review the complete reference script [here](https://github.com/critesjosh/aztec-sdk-starter/blob/main/src/shield.ts).