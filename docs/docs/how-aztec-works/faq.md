# Frequently Asked Questions

## General Questions

### What happens when I shield a token?

Aztec has an offchain UTXO architecture that functions like cash. Shielding a token functionally means depositing it into Aztec's Ethereum smart contracts, which then communicate with Aztec's offchain UTXO state system to generate cash-like encrypted notes.

The entire system accounts for a user's claim on Layer 1 tokens and assets via a fully encrypted state system (kind of like the briefcase of IOU's from Dumb and Dumber: https://www.youtube.com/watch?v=7GSXbgfKFWg). 

When assets are deposited to Aztec, withdrawn from Aztec, sent within Aztec, or exchanged for other assets via Aztec Connect, the state tree is updated with new encrypted notes. Because no one can peer inside a note except its owner, user identity and account balance is fully preserved.

For more information on Aztec's UTXO architecture, see this blog post: https://medium.com/aztec-protocol/fully-confidential-ethereum-transactions-aztec-networks-privacy-architecture-274f968b13d4

**Your transaction can go in two directions:**

1. Send zkETH/DAI to a username (@name) on zk.money. The recipient will get zkETH/DAI.

2. Send zkETH/DAI to any Ethereum address outside of zk.money - the wallet will receive regular ETH/DAI tokens. (Etherscan will show the funds were sent from Aztec’s contract, which doesn’t expose the sender 🕵️. If you have trouble seeing the funds on the public transaction log, please  click the "Internal Txn" tab).

![](https://user-images.githubusercontent.com/15220860/172720151-19349841-17b6-4861-a101-eacd5724144d.png)

---

### What happens when someone sends me zkETH/DAI?

1. **If your wallet is connected to zk.money -** you’ll receive zkETH/DAI directly to your account. You’ll be able to “un-shield” it back to L1 and have regular ETH or continue transacting with zkETH.

2. **If your wallet isn’t connected to zk.money** - you’ll be able to receive regular ETH/DAI directly into your wallet. The sender’s side will be anonymous. (Etherscan will show the funds were sent from an Aztec contract).

---

### Is there a transaction limit on zk.money?

Currently, users are limited to shielding **up to 5 $ETH or 10,000 $DAI at a time**.** 

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

###  Can I deposit and then immediately withdraw from Aztec?

We do not recommend this if you want privacy. If you deposit and immediately withdraw, observers might be able to deduce that the deposit and withdrawal belong to the same user, especially if the deposit/withdraw values are unique.

Ideally, you should wait at least until the rollup has processed a block of transactions before you withdraw. The longer you wait, the larger the anonymity set.

###  Are there ANY conditions under which I cannot withdraw my funds?

All of the information required to create an escape hatch withdrawal transaction can be extracted from published blocks on the Ethereum main-net.

There are two worst-case 'failure states' that would cause a loss of user funds:

1. The Ethereum protocol itself is compromised and we cannot rely on its consensus algorithm
2. There is a bug in our protocol that enables a hacker to steal funds

As long as both the Aztec and Ethereum protocol architectures are sound, it is not possible for user funds to be locked or stolen.

For more information about our bug bounty program and internal audit, please read [this article](https://medium.com/aztec-protocol/aztec-2-0-pre-launch-notes-aabad022808d).

### Are private send amounts fixed?

No. You can send any value internally within our rollup.


### Are deposit amounts fixed?

No. You can deposit any value into our system.

### Are withdrawal amounts fixed?

No, but it is recommended that your withdrawal transactions are either 0.1 Eth or 1 Eth. The more 'unique' your withdrawal value, the easier it is for observers to guess the deposit transactions that created your withdrawal transaction.

If you withdraw 0.1 Eth, for example, your withdraw tx looks like all of the other 0.1 Eth withdraw transactions that have occurred since your deposit transaction.

### Why can't I send Aztec transactions from an iPhone?

We are aware of an issue with the Safari web browser on iPhones. Our prover algorithms require 900MB of RAM, which iPhone Safari tabs do not currently allow for. We are working hard to resolve this issue and will update this FAQ once this is resolved.

---

## Rollup

### How long does it take for a transaction to settle?

The Rollup allows zk.money to batch a few transactions together (up to 112) and help users save on gas fees.

Aztec pays the network fees on behalf of the users - in that setup, users that broadcasted a transaction are paying a small share of the total cost according to their pro-rata share in the Rollup.

The more transactions are made, the faster each Roll will fill up, and completion time will be quicker. In case of slow periods, Aztec will subsidise remaining fees for the Rollup to ensure Rollup is sent at least every 6 hours. If users don't want to wait long, they may speed up transaction times by paying higher fees.

### Can the rollup provider censor my transactions and refuse to add them to a block?

The rollup provider cannot extract any information from your private transaction - to them, it looks like a list of random numbers. If the rollup provider wishes to censor a specific user, they are unable to do so.

### What happens if the rollup provider refuses to publish any blocks? Can I still withdraw?

Yes. Our 'escape hatch' mechanism enables withdrawals if the rollup provider goes down. For 2 hours per day, users can create 'escape hatch' proofs and send them directly to our rollup contract. This enables unconditional withdrawals from our system. Escape hatch proofs are time-consuming and expensive to create so this should only be used as a last resort.

### Will ordinary people be able to run a rollup?

Technically anyone can currently be a rollup provider for 2 out of every 10 hours right now. It is how the escape hatch mode works. We expect to decentralise further with community providers in ~3 months.

---
## Fees

### How does the protocol decide on the fee?

Currently, there are 28 slots in a rollup (we can increase this up to 128), the base fee is calculated as (1/28 of the proof verification gas costs + tx_type_feeConstant)  *** the gas price at the last rollup ***  a buffer of 1.2. The tx type fee constant is calculated as the fixed cost of calldata plus a constant depending on the assetId and the transaction type (shield, send, unshielded). For base fees, Aztec guarantees a rollup goes on-chain every 6 hours currently.

The rollup provider will automatically create a rollup proof whenever it has sufficient fees to cover an L1 transaction, or when the 6 hour block time has elapsed. The last fee slot is the cost for paying for 28 slots in the rollup and will trigger the rollup to create a proof instantly. The in-between fees levels are equivalent to paying for 10% and 50% of the slots in the rollup and reducing the 6 hour block time by 10% and 50% respectively. The idea here is to try and encourage users to pay a bit more so we can bring the block time down for everyone once we reach sustained throughput at say 4h.

One small side note, the rollup provider receives a refund for the gas cost of the L1 transaction, any surplus fees are owned by a contract and used to offset future rollups that may not be full.

We wanted to try and encourage a fee market and create a flow where Aztec does not make money from the fees, instead of a user overpaying, that payment helps to subsidise future transactions. As more rollup providers come online we will be changing this.

### How much do transactions cost?

- Deposit: 51,000 gas
- Internal send: 17,000 gas
- Withdraw: 5,000 gas for ETH to EOA 30,000 gas to contract.

### Do I pay any fees to Aztec for my zkETH/DAI transactions?

Aztec doesn’t make any money from transaction fees on the platform, surplus transaction fees accumulate on a smart contract and are used to subsided future transactions in busy periods with high gas prices.

---

### Have other questions?

Join our [Discord channel](https://discord.com/invite/UDtJr9u) and get an answer from the community.
