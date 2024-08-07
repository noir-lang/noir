# Fee Schedule

The [transaction fee](./specifying-gas-fee-info.md#transaction-fee) is comprised of a DA component, an L2 component, and an inclusion fee. The DA and L2 components are calculated by multiplying the gas consumed in each dimension by the respective `feePerGas` value. The inclusion fee is a fixed cost associated with the transaction, which is used to cover the cost of verifying the encompassing rollup proof on L1.

## DA Gas

DA gas is consumed to cover the costs associated with publishing data associated with a transaction.

These data include:

- new note hashes
- new nullifiers
- new l2 -> l1 message hashes
- new public data writes
- new logs
- protocol metadata (e.g. the amount of gas consumed, revert code, etc.)

The DA gas used is then calculated as:

```
DA_BYTES_PER_FIELD = 32
DA_GAS_PER_BYTE = 16
FIXED_DA_GAS = 512

# FIXED_DA_GAS covers the protocol metadata,
# which should remain less than 512/16 = 32 bytes

da_gas_per_field = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE

note_hash_gas = da_gas_per_field * (number of notes)
nullifier_gas = da_gas_per_field * (number of nullifiers)
l2_to_l1_message_gas = da_gas_per_field * (number of l2_to_l1_messages)

# public data writes specify a value and index
public_data_writes_gas = 2 * da_gas_per_field * (number of public_data_writes)

log_gas = DA_GAS_PER_BYTE * (unencrypted_log_preimages_length + encrypted_log_preimages_length)

da_gas_used = FIXED_DA_GAS +
                note_hash_gas +
                nullifier_gas +
                l2_to_l1_message_gas +
                public_data_writes_gas +
                log_gas +
                teardown_da_gas
```

:::note Non-zero `transaction_fees`
A side effect of the above calculation is that all transactions will have a non-zero `transaction_fee`.
:::

## L2 Gas

L2 gas is consumed to cover the costs associated with executing the public VM, proving the public VM circuit, and proving the public kernel circuit.

It is also consumed to perform fixed, mandatory computation that must be performed per transaction by the sequencer, regardless of what the transaction actually does; examples are TX validation and updating state roots in trees.

The public vm has an [instruction set](../public-vm/instruction-set.mdx) with opcode level gas metering to cover the cost of actions performed within the public VM.

Additionally, there is a fixed cost associated with each iteration of the public VM (i.e. the number of enqueued public function calls, plus 1 if there is a teardown function), which is used to cover the cost of proving the public VM circuit.

The L2 gas used is then calculated as:

```
FIXED_L2_GAS = 512
FIXED_AVM_STARTUP_L2_GAS = 1024


num_avm_invocations = (number of enqueued public function calls) +
                      (is there a teardown function ? 1 : 0)

l2_gas_used = FIXED_L2_GAS
                + FIXED_AVM_STARTUP_L2_GAS * num_avm_invocations
                + teardown_l2_gas
                + (gas reported as consumed by the public VM)
```

### L2 Gas from Private

Private execution also consumes L2 gas, because there is still work that needs to be performed by the sequencer correspondent to the private outputs, which is effectively L2 gas. The following operations performed in private execution will consume L2 gas:

- 32 L2 gas per note hash
- 64 L2 gas per nullifier
- 4 L2 gas per byte of logs (note encrypted, encrypted, and unencrypted)

## Max Inclusion Fee

Each transaction, and each block, has inescapable overhead costs associated with it which are not directly related to the amount of data or computation performed.

These costs include:

- verifying the private kernel proof of each transaction
- executing/proving the base/merge/root rollup circuits
  - includes verifying that every new nullifier is unique across the tx/block
  - includes processing l2->l1 messages of each transaction, even if they are empty (and thus have no DA gas cost)
  - includes ingesting l1->l2 messages that were posted during the previous block
  - injecting a public data write to levy the transaction fee on the `fee_payer`
- publishing the block header to the rollup contract on L1
  - includes verification of the rollup proof
  - includes insertion of the new root of the l2->l1 message tree into the L1 Outbox
  - consumes the pending messages in the L1 Inbox
- publishing the block header to DA

See [the l1 contracts section](../l1-smart-contracts/index.md) for more information on the L1 Inbox and Outbox.

Users cover these costs by [specifying an inclusion fee](./specifying-gas-fee-info.md#specifying-gas--fee-info), which is different from other parameters specified in that it is a fixed fee offered to the sequencer, denominated in [Fee Juice](./fee-juice.md).

Even though these line items will be the same for every transaction in a block, the **cost** to the sequencer will vary, particularly based on:

- congestion on L1
- prevailing price of proof generation

A price discovery mechanism is being developed to help users set the inclusion fee appropriately.
