---
title: Transaction Model
---

Aztec uses a [UTXO model](https://en.wikipedia.org/wiki/Unspent_transaction_output) for tracking ownership of assets. For the technical details of how this is managed, refer to the [Rollup Contract specification](../spec/rollup_contract) for how data is stored in the system and the [JoinSplit Circuit specification](../spec/join_split_circuit) for reference on how transactions are created and processed.

If you are trying to send an amount greater than the value of your largest [value note](./../glossary#value-notes), smaller notes must be merged into a value note greater than or equal to the value of the transaction. Merging value notes requires generating a proof for each note. The SDK handles merging notes automatically, but be aware that this may take a while if there are many notes that need to be merged.

:::caution

A max of two value notes can be used in a single transaction. If you have many small value notes that need to be aggregated into a larger note, you will have to pay transaction fees for each of these note merge transactions. The SDK will handle merging notes for you under the hood, but you will have to pay the corresponding transaction fees.

This means that it only makes sense to make deposits and transfers that are materially greater than the Aztec transaction fee.

:::

## Chaining notes

It is not required that notes are settled on Ethereum before they can be spent. This improves the user experience around sending notes. We refer to spending unsettled notes as "chaining" them together.

Here are some rules to keep in mind about chaining notes:

- Notes created from deposit cannot be chained from. Doing so will leak privacy. As there’s a link between two chained txs, people would be able to see that an L1 address is doing a defi deposit or withdrawing to another L1 address.

- A pending note created from transfer/defi/withdraw can be chained from, as long as the state is available locally. For example, if I have 1 note worth 1 ETH, I can create a transfer tx, sending Alice 0.2 ETH. And then I can continue to create another tx, sending Bob 0.3 ETH. But if I clear my local storage after this, or login from another device, I will see my balance is 1 ETH, but the spendable sum is 0. And I can’t spend the 0.5 ETH before the txs are settled.

- At most 1 pending note can be used in a chained tx. If I have 4 notes and each worth 0.5 ETH. And I send Alice 0.7 ETH, which merges 2 notes (0.5 + 0,5) and leaves me 1 pending note worth 0.3 ETH. And I send Bob 0.9 ETH, which also merges 2 notes (0.5 + 0,5) and leaves me another pending note with 0.1 ETH. I will not be able to use those two pending notes and send someone else 0.4 ETH. I will have to wait until at least one of them is settled. My spendable sum is 0.4 ETH, but the max spendable value is 0.3 ETH.
