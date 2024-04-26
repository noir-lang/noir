# Validity conditions

The _validity conditions_ of a transaction define when a [_transaction object_](./tx-object.md) is valid. Nodes should check the validity of a transaction when they receive it either directly or through the p2p pool, and if they find it to be invalid, should drop it immediately and not broadcast it.

In addition to being well-formed, the transaction object needs to pass the following checks:

<!--
Mike review: If we have written definitions for the various kinds of "`data`" described here, we should write the exact name of the struct (rather than repeating the word `data` for different kinds of data), and link to it, if possible.
- Also update/remove references to new contract data, in light of the new contract deployment ideas.
- TODO: also consider whether any checks relating to gas measurement and fees are needed (e.g. checking that the user-specified gas limit is above some baseline gas cost, given the data in the tx object).
-->

- **Proof is valid**: The `proof` for the given public `data` should be valid according to a protocol-wide verification key for the final private kernel circuit.
- **No duplicate nullifiers**: No `nullifier` in the transaction `data` should be already present in the nullifier tree.
- **No pending private function calls**: The `data` private call stack should be empty.
- **Valid historic data**: The tree roots in the block header of `data` must match the tree roots of a historical block in the chain.
- **Maximum block number not exceeded**: The transaction must be included in a block with height no greater than the value specified in `maxBlockNum` within the transaction's `data`.
- **Preimages must match commitments in `data`**: The expanded fields in the transaction object should match the commitments (hashes) to them in the public `data`.
  - The `encryptedLogs` should match the `encryptedLogsHash` and `encryptedLogPreimagesLength` in the transaction `data`.
  - The `unencryptedLogs` should match the `unencryptedLogsHash` and `unencryptedLogPreimagesLength` in the transaction `data`.
  - Each public call stack item in the transaction `data` should have a corresponding preimage in the `enqueuedPublicFunctionCalls`.
  - Each new contract data in transaction `data` should have a corresponding preimage in the `newContracts`.
- **Able to pay fee**: The [fee can be paid](../gas-and-fees/kernel-tracking.md#mempoolnode-validation).

Note that all checks but the last one are enforced by the base rollup circuit when the transaction is included in a block.
