---
title: Register Accounts
---

Please review the [Accounts overview](../../how-aztec-works/accounts) and [Add User](./add-user) pages if you haven't already.

As mentioned in the above pages, there is an important difference between the account nonce identifiers 0 and 1 in the Aztec account system. The privacy key associated with nonce 0 is used to decrypt notes, calculate balances and track transactions associated with an account. The signing keys associated with account nonce 1 are used to spend notes on behalf of a user.

Account with nonce 1 must be registered on the network before it can be used. The SDK makes it easy to register a new account using the `RegisterController`.

You can find the type definition of the RegisterController class [here](../types/RegisterController).

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

The `AssetValue` [type](../types/AssetValue) specifies the type and amount of an Aztec asset note.

`(await AztecSdk.getRegisterFees(deposit: AssetValue))[settlementTime: TxSettlementTime]` will help determine how much the transaction will cost. Settlement time can either be `NEXT_ROLLUP` or `INSTANT`. `INSTANT` is more expensive because it indicates you will pay for the rollup to settle "instantly" (on the order of minutes rather than hours) rather than waiting for it to fill up with other transactions. When there are more transactions in the rollup, users are splitting the cost among more transactions so each one is cheaper.

Once the `RegisterController` has been created, you can use it to deposit funds, create proofs, and sign and send transactions. For example:

```ts
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
