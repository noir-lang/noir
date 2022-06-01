---
title: Register Accounts
---

Please review the [Accounts overview](../../how-aztec-works/accounts) and [Add User](./add-user) pages if you haven't already.

As mentioned in the above pages, there is an important difference between the account nonce identiers 0 and 1 in the Aztec account system. The privacy key associated with nonce 0 is used to decrypt notes, calculate balances and track transactions associated with an account. The signing keys associated with account nonce 1 are used to spend notes on behalf of a user.

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

You can find an example script that calls this function [here](https://github.com/critesjosh/aztec-sdk-starter/blob/main/src/registerAccount.ts).