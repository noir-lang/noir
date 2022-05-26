---
title: FAQ
---

## Frequently Asked Questions

### What happens when I shield a token?

Your tokens simply get a zkSNARK “shield”, which anonymises any related data under that state.

**Your transaction can go in two directions:**

1. Send zkETH/DAI to a username (@name) on zk.money. The recipient will get zkETH/DAI.

2. Send zkETH/DAI to any Ethereum address outside of zk.money - the wallet will receive regular ETH/DAI tokens. (Etherscan will show the funds were sent from Aztec’s contract, which doesn’t expose the sender 🕵️).

---

### What happens when someone sends me zkETH/DAI?

1. **If your wallet is connected to zk.money -** you’ll receive zkETH/DAI directly to your account. You’ll be able to “un-shield” it back to L1 and have regular ETH or continue transacting with zkETH.

2. **If your wallet isn’t connected to zk.money** - you’ll be able to receive regular ETH/DAI directly into your wallet. The sender’s side will be anonymous. (Etherscan will show the funds were sent from an Aztec contract).

---

### Is there a transaction limit on zk.money?

Currently, users are limited to shielding and sending **up to $100,000 at a time**.** 

---

### What if my deposit fails mid-way through?

**If your deposit fails mid-way through or you accidentally close the tab, your funds are safe on the contract.**

You can retry your transaction by re-shielding via the shield form. The text under the input box will show any pending deposit balance.

![zkmoney1](/img/zkmoney1.png)

---

### How can I track my transactions?

**You’ll be able to see and track your transactions' status on the dashboard.**

And you can also track your transactions in real-time in the Aztec explorer via the transaction link. If you shield to a username that isn’t your own, the amount and recipient will not show in the transaction history, because the data is encrypted.

---

### Is zk.money decentralized?

zk-money is powered by Aztec. The current release of Aztec's L2 Rollup technology is run with Aztec as the sole rollup provider, with more rollup providers onboarding later this year. At that point, the system will be decentralised. Currently, users rely on Aztec to relay rolled up transactions to the chain. In case Aztec suddenly disappears, there is an emergency mode to allow users to withdraw funds from the system directly from the contract. [More info here](https://developers.aztec.network/###/A%20Private%20Layer%202/zkAssets/emergencyWithdraw).

---

###  If I send somebody a private transaction, will they know I am the sender? Will they be able to read my balance? 

No. In the future, you will be able to choose to reveal your identity to the recipient if required, but this feature is still under development.

---

###  How quickly can I withdraw? 

Initially, we will be publishing rollup blocks every 4 hours. This time interval will decrease with increased use.

If you wish to immediately withdraw, you can increase your transaction fee to pay for a rollup block to be immediately constructed and published. It will take approximately 10 mins for a block to be published on main-net and the withdrawal finalised.

---

###  Can I deposit and then immediately withdraw from Aztec? 

We do not recommend this if you want privacy. If you deposit and immediately withdraw, observers might be able to deduce that the deposit and withdrawal belong to the same user, especially if the deposit/withdraw values are unique.

Ideally, you should wait at least until the rollup has processed a block of transactions before you withdraw. The longer you wait, the larger the anonymity set.

---

###  Are there ANY conditions under which I cannot withdraw my funds? 

All of the information required to create an escape hatch withdrawal transaction can be extracted from published blocks on the Ethereum main-net.

There are two worst-case 'failure states' that would cause a loss of user funds:

1. The Ethereum protocol itself is compromised and we cannot rely on its consensus algorithm
2. There is a bug in our protocol that enables a hacker to steal funds

As long as both the Aztec and Ethereum protocol architectures are sound, it is not possible for user funds to be locked or stolen.

For more information about our bug bounty program and internal audit, please read [this article](https://medium.com/aztec-protocol/aztec-2-0-pre-launch-notes-aabad022808d).

---

###  Are private send amounts fixed? 

No. You can send any value internally within our rollup.

---

###  Are deposit amounts fixed? 

No. You can deposit any value into our system.

---

###  Are withdrawal amounts fixed? 

No, but it is recommended that your withdrawal transactions are either 0.1 Eth or 1 Eth. The more 'unique' your withdrawal value, the easier it is for observers to guess the deposit transactions that created your withdrawal transaction.

If you withdraw 0.1 Eth, for example, your withdraw tx looks like all of the other 0.1 Eth withdraw transactions that have occurred since your deposit transaction.

---

###  Why can't I send Aztec transactions from an iPhone? 

We are aware of an issue with the Safari web browser on iPhones. Our prover algorithms require 900MB of RAM, which iPhone Safari tabs do not currently allow for. We are working hard to resolve this issue and will update this FAQ once this is resolved.

---

### Have other questions?

Join our [Discord channel](https://discord.com/invite/UDtJr9u) and get an answer from the community