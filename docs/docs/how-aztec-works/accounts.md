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

Users in Aztec will use the main account to receive notes and decrypt balances and the signer (or spending key) to spend notes or initiate bridged Ethereum interactions.

### Account

The privacy account is the first account that is generated for an Aztec user.

The private key associated with this account can be used to decrypt notes. The private key can also be used to register a distinct spending key. This allows for account abstraction by creating a separation between the key required to decrypt notes (privacy key) and the key required to spend notes (spending key). If a spending has not been registered, the account private key can be used.

Accounts can be identified by their alias or their public key. You can read more about aliases below.

### Spending keys (signer)

An account should register a signer with a new spending key on the network in order to take advantage of account abstraction.

When an account is first registered, you can pick a human-readable alias, a spending key and a recovery key. If the spending key is lost, a recovery flow can be initiated by the recovery account specified when the new spending key was registered (account recovery).

Registering a spending key has an associated fee as it typically includes a token (or ETH) deposit and posts transactions to the network.

You can add as many spending keys to an account as you want. This allows you to spend notes from the same account from multiple devices without having to share sensitive private keys across devices.

### Account Registration

To register a new account, you need to choose an alias and a new spending public key. Optionally, you can include a recovery account public key and a deposit.

Generally, an account with a registered spending key is considered safer than account that only uses the default account keys. An account without a spending key uses default account private key for note decryption as well as spending notes. When a spending key is registered, the default private key can only be used for decrypting notes and spending must be done with a registered spending key.

Most users will typically use an account with a registered spending key and are thus considered "safe". There are use cases (airdrops) where you might want to use an account that has not yet registered a spending key and is using the default account key for both note decryption and spending. So it is possible to use the system without registering your account.

When you use an unregistered account, your notes are marked as spendable by the account key. It's the sender that defines wether notes are marked spendable with the account key. A sender can check whether an account has registered spending keys before specifying the spending key.

You cannot mix the spending of these notes. You can send unspent notes from the default account to yourself, but marked as spendable by the signing key.

The SDK tries to abstract much of this complexity away and presents everything to a developer as if this notion does not exist (e.g. the account balance is the sum of all notes regardless of registered or not).

If you want to know exactly what you can spend in one transaction, you have to tell the SDK whether your interested in the unregistered or registered balances.

When actually creating the zero knowledge proof, the SDK infers which balance you're drawing from based on whether you give it a spending key or the account key.

### Account Alias

The main privacy account public key is associated with a human-readable alias when the account registers a new signing key (see below). The alias can be anything (20 alphanumeric, lowercase characters or less) as long as it hasn't been claimed yet.

Do not forget your alias. If you forget your alias you will not be able to share it to make it easy for them to send you asset notes.

There is no way to recover a forgotten alias, but you can register a new account with a new alias and transfer your notes to the new account.

If you forget your alias you can still transfer and withdraw asset notes.

### Account Migration

Account migration allows you to keep your alias and just update your account (privacy) keys. This will update the public key associated with your alias as well as the key that is used to decrypt your account notes. This can only be done 1 time.

### Account Recovery

If you lose access to all of your spending keys for an account, the designated recovery account can help you recover access and register a new spending key that you have access to.

This recovery information is created and registered with the account during the account registration step.

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