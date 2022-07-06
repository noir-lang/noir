# zk.money Fees

## Fee Calculation

Fees on zk.money are calculated depending on the user action.

There are broadly two types of actions:

1) simple payment transactions (deposits, withdrawals, and internal sends)
2) DeFi interactions

## Simple Payments

The first class of simple transactions has the basic cost of being included in a rollup.

Being included in a rollup also has two components:

a) rollup verification costs

Total rollup verification cost is approximately 550,000 gas. A transaction's share of total verification costs is calculated by dividing this total rollup verification by the number of transactions. For an 896 transaction rollup, the base fee is 614 gas.

b) call data costs

Call data has to be posted to Ethereum to update Aztec's state tree with the new offchain system state. The cost of posting call data is approximately 15,376 gas, less than the base transaction cost on Ethereum.

**This means the cost of executing a private transaction is always cheaper than doing a public transaction on Ethereum.**

For deposits, withdrawals, and internal transfers, the story stops there.

## Defi Interactions

Defi interactions are actually comprised of multiple simple transactions, as they are structured like swaps: one or two input assets are sent from the Aztec rollup to another Layer 1 protocol, and then one or two output assets are received. Therefore, the rollup verification cost is typically double the cost of a simple transaction (essentially because they are comprised of 2 transactions).

Call data costs--the cost to update state--remain the same.

However, DeFi interactions also carry a Layer 1 execution cost--the cost of processing a DeFi transaction on Layer 1. DeFi interactions are batched to different sizes depending on the bridge.

Therefore, the cost of Layer 1 execution costs for a given contract interaction is t/n, t being the total cost of Layer 1 execution and n being the size of transaction batch processed by that particular bridge. Information on individual batch sizes can be found when determining fees for zk.money transactions.


## Transaction Speed

As mentioned in the zk.money User Guide, there are three potential speeds you might see for simple transactions and DeFi transactions:

1) Batched
2) Fast Track (only available for DeFi transactions)
3) Instant

Batched transactions fully split costs for both the rollup and, if there's a DeFi transaction involved, the DeFi transaction. Transactions in the DeFi batch enter the rollup once the batch is full (e.g. all 10 of the Element vault entry transactions in a batch are full), and the rollup is posted to Ethereum Layer 1 once it is full.

Fast Track transactions accelerate the "filling" of a DeFi batch, allowing users to pay the remainder of the batch costs for inclusion in the next Aztec rollup.

Instant transactions both fill a DeFi batch and fill a rollup, ensuring the transaction (along with others in the DeFi batch and overall rollup) goes to Layer 1 Ethereum as soon as possible.

For all transactions, the Aztec rollup includes a small markup to the total fee as a buffer in case Ethereum gas costs experience a sharp rise between the time an order is executed and the time the rollup is published to Layer 1.

For more information on how we scale our rollup, refer to this Medium post: https://medium.com/aztec-protocol/dollars-and-sense-cheap-privacy-with-aztec-connect-f35db037a04

