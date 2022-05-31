---
title: Accounts
---

Accounts in Aztec.

## A Technical Primer on Accounts

Accounts in Aztec work differently than accounts in Ethereum.

There are two main parts to each Aztec account, the privacy account associated with nonce 0 and the spending account associated with nonce 1. Both of these accounts have the same public key and are differentiated by the nonce.

Additionally, Aztec uses a different curve than Ethereum for SNARK efficient operations. This means that you cannot use an Ethereum private key directly for signing Aztec transactions or a public key for deriving an account address. Specifically, Aztec uses the Grumpkin curve, see [the yellow paper](https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations) for more information.

In [zk.money](https://zk.money), Aztec accounts are generated using Ethereum accounts by having a user sign a message and deriving the Aztec keys from the signed message. Different messages are used to generate different keys.

### Privacy Account

The privacy account is the first account that is generated for an Aztec user.

The private key associated with this account can be used to decrypt notes for both the privacy account (nonce 0) and the spending account (nonce 1). The private key is used to spend notes associated with the privacy account and register a spending account. Typically the privacy account does not handle funds and is used only to create a spending account and decrypt notes. This allows for basic account abstractions by creating a separation of the key required to decrypt notes and the key required to spend notes.

### Spending Account

Spending accounts are registered with a human-readable alias, a spending key and a recovery key.

The spending account is typically the account where users will receive and spend notes (this is how it is done in [zk.money](https://zk.money)). The spending account has the same public key as the privacy account but is differentiated by the nonce. Notes associated with the spending account can be spent by the spending key that is defined when the spending account is registered. If the spending key is lost, a recovery flow can be initiated by the recovery account specified when the spending account was registered.

The spending account is associated with a human-readable alias. The alias can be anything as long as it hasn't been claimed yet.

Registering a spending account has an associated fee as it typically includes a token (or ETH) deposit and is posting transactions to the network.

For a technical overview of accounts on Aztec including code examples using the SDK, see [this reference repository](https://github.com/critesjosh/aztec-sdk-starter).

## Frequently Asked Questions

### What happens if I lose my Aztec account private key?

Your Aztec account private keys are derived from an Ethereum signature we ask you to sign when you register with us. As long as you still control your original Ethereum account you can re-derive your Aztec account keys.

---

### What happens if I lose my Aztec account private key AND my Ethereum account private key?

At the current time, your funds would be lost. Our protocol architecture supports Aztec account social recovery but implementation into our front-end software is still under development.

---

### What is the zk.money username for?

zk.money username/alias lets other users easily lookup your encryption public key so they can send you assets. This name has to be unique and is limited to 20 characters, lowercase, alphanumeric. Please note that this isn’t an ENS domain.

---

### I've registered to the platform, but zk.money prompts registration once again when I try to log in again?

Please follow these instructions:

**Step 1:** Clear browser cache. For Chrome, this is the link: chrome://settings/cookies/detail?site=zk.money

**Step 2:** Make sure you are signing in with the Metamask account you used to register zkmoney username.

> 💡 Sign in with Metamask account you used to register your zkmoney username (You might have previously used a different Metamask account for funding/depositing).

**Step 3:**

- If it shows "Claim Username," it means your previous deposit has not been used. Claim Username to proceed.
- If it asks you to "Deposit", it is likely we have a bug. Don't proceed and message the support team at [Discord](https://discord.gg/9TaSvc8f7r).