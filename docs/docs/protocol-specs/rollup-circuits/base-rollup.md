---
title: Base Rollup
---

The base rollup circuit is the most complex of the rollup circuits, as it has to interpret the output data of a kernel proof and perform the state updates and transaction validation. While this makes the data structures complex to follow, the goal of the circuit is fairly straight forward:

Take `BaseRollupInputs` as an input value, and transform it to `BaseOrMergeRollupPublicInputs` as an output value while making sure that the validity conditions are met.

```mermaid
graph LR
A[BaseRollupInputs] --> C[BaseRollupCircuit] --> B[BaseOrMergeRollupPublicInputs]
```

## Overview

Below is a subset of the figure from [earlier](./index.md) (granted, not much is removed). The figure shows the data structures related to the Base Rollup circuit.

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
    coinbase: EthAddress
    fee_recipient: Address
    gas_fees.fees_per_da_gas: Fr
    gas_fees.fees_per_l2_gas: Fr
}

class ContentCommitment {
    tx_tree_height: Fr
    txs_hash: Fr[2]
    in_hash: Fr[2]
    out_hash: Fr[2]
}

class Header {
    last_archive: Snapshot
    content_commitment: ContentCommitment
    state: StateReference
    global_variables: GlobalVariables
    total_fees: Fr
}
Header *.. Body : txs_hash
Header *-- ContentCommitment: content_commitment
Header *-- StateReference : state
Header *-- GlobalVariables : global_variables

class ContractData {
    leaf: Fr
    address: Address
    portal: EthAddress
}

class Logs {
    private: EncryptedLogs
    public: UnencryptedLogs
}

class PublicDataWrite {
    index: Fr
    value: Fr
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
    tx_effects: List~TxEffect~
}
Body *-- "m" TxEffect

class ProvenBlock {
    archive: Snapshot
    header: Header
    body: Body
}

ProvenBlock *-- Header : header
ProvenBlock *-- Body : body

class ConstantRollupData {
  last_archive: Snapshot
  base_rollup_vk_hash: Fr,
  merge_rollup_vk_hash: Fr,
  global_variables: GlobalVariables
}
ConstantRollupData *-- GlobalVariables : global_variables

class PublicDataUpdateRequest {
    index: Fr
    old_value: Fr
    new_value: Fr
}

class PublicDataRead {
    index: Fr
    value: Fr
}

class CombinedAccumulatedData {
    read_requests: List~Fr~
    pending_read_requests: List~Fr~
    note_hashes: List~Fr~
    nullifiers: List~Fr~
    nullified_note_hashes: List~Fr~

    l2_to_l1_messages: List~Fr~
    public_update_requests: List~PublicDataUpdateRequest~
    public_reads: List~PublicDataRead~
    logs: Logs

    private_call_stack: List~CallRequest~
    public_call_stack: List~CallRequest~
    start_public_data_root: Fr
    end_public_data_root: Fr

    gas_used.da_gas: u32
    gas_used.l2_gas: u32
}
CombinedAccumulatedData *-- "m" PublicDataUpdateRequest: public_update_requests
CombinedAccumulatedData *-- "m" PublicDataRead: public_reads
CombinedAccumulatedData *-- Logs : logs

class TxContext {
    chain_id: Fr
    version: Fr
    gas_settings: GasSettings
}

TxContext *-- GasSettings : gas_settings

class CombinedConstantData {
    historical_header: Header
    tx_context: TxContext
    global_variables: GlobalVariables
}
CombinedConstantData *-- Header : historical_header
CombinedConstantData *-- TxContext : tx_context
CombinedConstantData *-- GlobalVariables : global_variables

class GasSettings {
    da.gas_limit: u32
    da.teardown_gas_limit: u32
    da.max_fee_per_gas: Fr
    l1.gas_limit: u32
    l1.teardown_gas_limit: u32
    l1.max_fee_per_gas: Fr
    l2.gas_limit: u32
    l2.teardown_gas_limit: u32
    l2.max_fee_per_gas: Fr
    inclusion_fee: Fr
}

class KernelPublicInputs {
  is_private: bool
  end: CombinedAccumulatedData
  constants: CombinedConstantData
}
KernelPublicInputs *-- CombinedAccumulatedData : end
KernelPublicInputs *-- CombinedConstantData : constants

class KernelData {
  proof: Proof
  public_inputs: KernelPublicInputs
}
KernelData *-- KernelPublicInputs : public_inputs

class StateDiffHints {
  nullifier_predecessor_preimages: List~NullifierLeafPreimage~
  nullifier_predecessor_membership_witnesses: List~NullifierMembershipWitness~
  sorted_nullifiers: List~Fr~
  sorted_nullifier_indexes: List~Fr~
  note_hash_subtree_sibling_path: List~Fr~,
  nullifier_subtree_sibling_path: List~Fr~,
  contract_subtree_sibling_path: List~Fr~,
  public_data_sibling_path: List~Fr~,
}

class BaseRollupInputs {
  historical_header_membership_witnesses: HeaderMembershipWitness
  kernel_data: KernelData
  partial: PartialStateReference
  state_diff_hints: StateDiffHints
}
BaseRollupInputs *-- KernelData : kernelData
BaseRollupInputs *-- PartialStateReference : partial
BaseRollupInputs *-- StateDiffHints : state_diff_hints
BaseRollupInputs *-- ConstantRollupData : constants

class BaseOrMergeRollupPublicInputs {
    type: Fr
    height_in_block_tree: Fr
    txs_hash: Fr[2]
    out_hash: Fr[2]
    constants: ConstantRollupData
    start: PartialStateReference
    end: PartialStateReference
}
BaseOrMergeRollupPublicInputs *-- ConstantRollupData : constants
BaseOrMergeRollupPublicInputs *-- PartialStateReference : start
BaseOrMergeRollupPublicInputs *-- PartialStateReference : end
```

:::warning TODO  
Fee structs and contract deployment structs will need to be revised, in line with newer ideas.  
:::

### Validity Conditions

```python
def BaseRollupCircuit(
  state_diff_hints: StateDiffHints,
  historical_header_membership_witnesses: HeaderMembershipWitness,
  kernel_data: KernelData,
  partial: PartialStateReference,
  constants: ConstantRollupData,
) -> BaseOrMergeRollupPublicInputs:

  public_data_tree_root = partial.public_data_tree
  tx_hash, contracts, public_data_tree_root = kernel_checks(
    kernel_data,
    constants,
    public_data_tree_root,
    historical_header_membership_witnesses,
  )

  note_hash_subtree = MerkleTree(kernel_data.public_inputs.end.note_hashes)
  note_hash_snapshot = merkle_insertion(
    partial.note_hash_tree.root,
    note_hash_subtree.root,
    state_diff_hints.note_hash_subtree_sibling_path,
    NOTE_HASH_SUBTREE_HEIGHT,
    NOTE_HASH_TREE_HEIGHT,
  )

  # We can use the sorted nullifiers to simplify batch-insertion
  # The sorting can be checked with a permutation
  nullifier_snapshot = successor_merkle_batch_insertion(
    partial.nullifier_tree.root,
    kernel_data.public_inputs.end.nullifiers,
    state_diff_hints.sorted_nullifiers,
    state_diff_hints.sorted_nullifier_indexes,
    state_diff_hints.nullifier_subtree_sibling_path,
    state_diff.nullifier_predecessor_preimages,
    state_diff.nullifier_predecessor_membership_witnesses,
    NULLIFIER_SUBTREE_HEIGHT,
    NULLIFIER_TREE_HEIGHT,
  )

  contract_sub_tree = MerkleTree(contracts)
  contract_snapshot = merkle_insertion(
    partial.note_hash_tree.root,
    note_hash_subtree.root,
    state_diff_hints.contract_subtree_sibling_path,
    CONTRACTS_SUBTREE_HEIGHT,
    CONTRACTS_TREE_HEIGHT,
  )

  out_hash = SHA256(kernel_data.public_inputs.end.l2_to_l1_messages)

  return BaseOrMergeRollupPublicInputs(
    type=0,
    height_in_block_tree=0,
    aggregation_object=
    txs_hash=tx_hash
    out_hash=out_hash
    start=partial,
    end=PartialStateReference(
      note_hash_tree=note_hash_snapshot,
      nullifier_tree=nullifier_snapshot,
      contract_tree=contract_snapshot,
      public_data_tree=public_data_tree_root,
    ),
  )

def kernel_checks(
  kernel: KernelData,
  constants: ConstantRollupData,
  public_data_tree_root: Fr,
  historical_header_membership_witness: HeaderMembershipWitness
) -> (Fr[2], Fr[], Fr):
  assert public_data_tree_root == kernel.public_inputs.end.start_public_data_root
  assert kernel.proof.verify(kernel.public_inputs)

  tx_context = kernel.public_inputs.constants.tx_context
  assert tx_context.chain_id == constants.global_variables.chain_id
  assert tx_context.version == constants.global_variables.version

  assert len(kernel.public_inputs.end.private_call_stack) == 0
  assert len(kernel.public_inputs.end.public_call_stack) == 0

  assert merkle_inclusion(
    kernel.constants.historical_header.hash(),
    kernel.constants.historical_header.global_variables.block_number,
    historical_header_membership_witness,
    constants.last_archive
  )

  contracts = []
  contract_datas = []
  for preimage in kernel.public_inputs.end.contracts:
    to_push = preimage.hash() if preimage.address == 0 else 0:
    contracts.push(to_push)
    contract_datas.push(ContractData(to_push, preimage.address, preimage.portal))

  tx_hash = SHA256(
    kernel.public_inputs.end.note_hashes |
    kernel.public_inputs.end.nullifiers |
    contract_datas | 
    kernel.public_inputs.end.public_data_writes |
    kernel.public_inputs.end.l2_to_l1_messages
  )
  return (tx_hash, contracts, kernel.public_inputs.end.end_public_data_root)
```
