# Public Data Tree

The Public Data tree is an [indexed Merkle tree](./tree-implementations.md#indexed-merkle-trees) that stores public-state key-value data. Each item stored in the tree is a key-value pair, where both key and value are 254-bit altBN-254 scalar field elements. Items are sorted based on their key, so each indexed tree leaf contains a tuple with the key, the value, the next higher key, and the index in the tree for the next higher key. This tree is part of the global state, and is updated by the sequencer during the execution of public functions.

The Public Data tree is implemented using an indexed Merkle tree instead of a sparse Merkle tree in order to reduce the tree height. A lower height means shorter membership proofs.

Keys in the Public Data tree are [siloed](./tree-implementations.md#siloing-leaves) using the contract address, to prevent a contract from overwriting public state for another contract.

```
fn compute_siloed_public_data_item(key, value, contract):
  let siloed_key = hash([contract, key], PUBLIC_DATA_LEAF)
  return [siloed_key, value]
```

When reading a key from the Public Data tree, the key may or may not be present. If the key is not present, then a non-membership proof is produced, and the value is assumed to be zero. When a key is written to, either a new node is appended to the tree if the key was not present, or its value is overwritten if it was.

Public functions can read from or write to the Public Data tree by emitting `contract_storage_read` and `contract_storage_update_requests` in the `PublicCircuitPublicInputs`. The Kernel circuit then siloes these requests per contract.

Contracts can store arbitrary data at a given key, which is always stored as a single field element. Applications are responsible for interpreting this data. Should an application need to store data larger than a single field element, they are responsible for partitioning it across multiple keys.
