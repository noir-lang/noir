---
title: Register
---

Register an Aztec spending key, alias and (optional) recovery key.

Please review the [Accounts overview](../../how-aztec-works/accounts) and [Add User](./add-account) pages if you haven't already.

As mentioned in the above pages, there is an important difference between the account (privacy) keys and spending keys. The account key is used to decrypt notes, calculate balances and track transactions associated with an account. The spending key associated with an account is used to spend notes associated with an account.

A spending key must be registered on the network before it can be used. The SDK makes it easy to register a new spending key using the `RegisterController`.

You can find the interface for the `RegisterController` class [here](../types/sdk/RegisterController).

## Setup

You can find the type definition of the RegisterController class [here](../types/sdk/RegisterController).

```ts
AztecSdk.createRegisterController(
    userId: GrumpkinAddress, 
    alias: string, 
    accountPrivateKey: Buffer, 
    spendingPublicKey: GrumpkinAddress, 
    recoveryPublicKey: GrumpkinAddress | undefined, 
    deposit: AssetValue, 
    fee: AssetValue, 
    depositor: EthAddress, 
    feePayer?: FeePayer, 
    provider?: EthereumProvider) : Promise<RegisterController>;
```

### Controller Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key of the account registering the new signing key. |
| alias | string | The alias to register the new account with. This is typically a human-readable, easy to remember identifier. |
| accountPrivateKey | Buffer | The account private key. |
| spendingPublicKey | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | The public key for the new account. Users must remember the corresponding private key (or the derivation method). |
| recoveryPublicKey | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) or `undefined` | An optional recovery key that allows the account to be recovered if the spending key is lost. |
| deposit | [AssetValue](../types/barretenberg/AssetValue) | The `assetId` (number) and `value` (bigint) to deposit. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | The network fee for registering the account. |
| depositor | [EthAddress](../types/barretenberg/EthAddress) | The Ethereum account from which to deposit the funds. |
| feePayer? | [FeePayer](../types/sdk/FeePayer) | Optional account to pay the registration fee if the fee is to be paid with funds in Aztec instead of Ethereum. Relevant when depositing assets that can't be used for fees in Aztec (deposit wstETH, pay fees with zkETH) |
| provider? | [EthereumProvider](../types/barretenberg/EthereumProvider) | Optional Ethereum Provider. When unspecified it defaults to the provider used in setup (`createAztecSdk`). |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [RegisterController](../types/sdk/RegisterController) | A user instance with apis bound to the user's account id. |

### Calculating Fees

Registering an account costs a fee to cover gas costs for posting calldata on Ethereum as well as to prevent alias squatting.

`(await AztecSdk.getRegisterFees(deposit: AssetValue))[settlementTime: TxSettlementTime]` will help determine how much the transaction will cost. Settlement time can either be `NEXT_ROLLUP` or `INSTANT`. `INSTANT` is more expensive because it indicates you will pay for the rollup to settle "instantly" (on the order of minutes rather than hours) rather than waiting for it to fill up with other transactions. When there are more transactions in the rollup, users are splitting the cost among more transactions so each one is cheaper.

Note that the cost of `INSTANT` is the same regardless of how full the rollup is at the time of requesting the quote. The settlement time is inferred from the fee a user pays, it is not explicitly sent to the controller.

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

// check if there is a pending deposit
if ((await controller.getPendingFunds()) < tokenQuantity) {
    await controller.depositFundsToContract();
    await controller.awaitDepositFundsToContract();
}

await controller.createProof();
await controller.sign();
await controller.send();
```

You can find the full example script that calls this method [here](https://github.com/critesjosh/aztec-sdk-starter/blob/mainnet-fork/src/latest/registerAccount.ts).
