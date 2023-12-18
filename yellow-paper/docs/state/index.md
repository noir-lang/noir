---
title: State
sidebar_position: 10
---

# State

Global state in the Aztec Network is represented by a set of Merkle trees: the [Note Hash tree](./note_hash_tree.md), [Nullifier tree](./nullifier_tree.md), and [Public Data tree](./public_data_tree.md) reflect the latest state of the chain, while the L1 to L2 message tree allows for [cross-chain communication](../contracts/#l2-outbox) and the [Archive](./archive.md) allows for historical state access.

Merkle trees are either 
- [append-only](./tree_impls.md#append-only-merkle-trees), for data where we only require inclusion proofs or 
- [indexed](./tree_impls.md#indexed-merkle-trees) for storing data that requires proofs of non-membership.

```mermaid
classDiagram
direction TB


class PartialStateReference {
    note_hash_tree: Snapshot
    nullifier_tree: Snapshot
    contract_tree: Snapshot
    public_data_tree: Snapshot
}

class StateReference {
    l1_to_l2_message_tree: Snapshot
    partial: PartialStateReference
}
StateReference *-- PartialStateReference: partial

class GlobalVariables {
    block_number: Fr
    timestamp: Fr
    version: Fr
    chain_id: Fr
    coinbase: Address
}

class Header {
    last_archive: Snapshot
    content_hash: Fr[2]
    state: StateReference
    global_variables: GlobalVariables
}
Header *.. Body : content_hash
Header *-- StateReference : state
Header *-- GlobalVariables : global_variables

class Logs {
    private: EncryptedLogs
    public: UnencryptedLogs
}

class PublicDataWrite {
    index: Fr
    value: Fr
}

class ContractData {
    leaf: Fr
    address: Address
    portal: EthAddress
}

class TxEffect {
    note_hashes: List~Fr~
    nullifiers: List~Fr~
    l2_to_l1_msgs: List~Fr~
    contracts: List~ContractData~
    public_writes: List~PublicDataWrite~
    logs: Logs
}
TxEffect *-- "m" ContractData: contracts
TxEffect *-- "m" PublicDataWrite: public_writes
TxEffect *-- Logs : logs

class Body {
    l1_to_l2_messages: List~Fr~
    tx_effects: List~TxEffect~
}
Body *-- "m" TxEffect

class Archive {
  type: AppendOnlyMerkleTree
  leaves: List~Header~
}
Archive *.. "m" Header : leaves


class NoteHashTree {
  type: AppendOnlyMerkleTree
  leaves: List~Fr~
}

class NewContractData {
    function_tree_root: Fr
    address: Address
    portal: EthAddress
}

class ContractTree {
  type: AppendOnlyMerkleTree
  leaves: List~NewContractData~
}
ContractTree *.. "m" NewContractData : leaves 

class PublicDataPreimage {
  key: Fr
  value: Fr
  successor_index: Fr
  successor_value: Fr
}

class PublicDataTree {
  type: SuccessorMerkleTree
  leaves: List~PublicDataPreimage~
}
PublicDataTree *.. "m" PublicDataPreimage : leaves 

class L1ToL2MessageTree {
  type: AppendOnlyMerkleTree
  leaves: List~Fr~
}

class NullifierPreimage {
  value: Fr
  successor_index: Fr
  successor_value: Fr
}

class NullifierTree {
  type: SuccessorMerkleTree
  leaves: List~NullifierPreimage~
}
NullifierTree *.. "m" NullifierPreimage : leaves

class State { 
  archive: Archive
  note_hash_tree: NoteHashTree
  nullifier_tree: NullifierTree
  public_data_tree: PublicDataTree
  contract_tree: ContractTree
  l1_to_l2_message_tree: L1ToL2MessageTree
}
State *-- L1ToL2MessageTree : l1_to_l2_message_tree
State *-- Archive : archive
State *-- NoteHashTree : note_hash_tree
State *-- NullifierTree : nullifier_tree
State *-- PublicDataTree : public_data_tree
State *-- ContractTree : contract_tree
```


import DocCardList from '@theme/DocCardList';

<DocCardList />
