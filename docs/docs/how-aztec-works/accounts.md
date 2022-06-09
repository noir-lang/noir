---
title: Accounts
---

Accounts in Aztec.

## A Technical Primer on Accounts

Accounts in Aztec work differently than accounts in Ethereum.

There are two main parts to each Aztec account, the account and the signer. The account is associated with a privacy key that can be used to decrypt account notes (assets on Aztec). The signer is associated with a spending key (or signing key) which can be used to send notes on the Aztec network.

Aztec uses a different curve than Ethereum for SNARK efficient operations. This means that you cannot use an Ethereum private key directly for signing Aztec transactions or a public key for deriving an account address. Specifically, Aztec uses the Grumpkin curve, see [the yellow paper](https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations) for more information.

In [zk.money](https://zk.money), Aztec accounts are generated using Ethereum accounts by having a user sign a message and deriving the Aztec keys from the signed message. This ensures that as long as someone has access to their Ethereum account, they will be able to access their Aztec account by signing a message. Different messages are used to generate different keys (account decryption key and spending key).

## Users And Accounts

Users in Aztec will use the main account to receive notes and decrypt balances and the signer to spend notes or initiate bridged Ethereum interactions.

### Account

The privacy account is the first account that is generated for an Aztec user.

The private key associated with this account can be used to decrypt notes. The private key can also be used to register a distinct spending key. This allows for account abstraction by creating a separation between the key required to decrypt notes (privacy key) and the key required to spend notes (spending key). If a spending has not been registered, the account private key can be used.

The main privacy account public key is associated with a human-readable alias when the account registers a new signing key (see below). The alias can be anything as long as it hasn't been claimed yet.

### Signer

An account should register a signer with a new spending key on the network in order to take advantage of account abstraction.

Signers are registered with a human-readable alias, a spending key and a recovery key. If the spending key is lost, a recovery flow can be initiated by the recovery account specified when the new spending key was registered.

Registering a spending key has an associated fee as it typically includes a token (or ETH) deposit and posts transactions to the network.

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