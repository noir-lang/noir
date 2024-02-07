---
title: Rollup Circuits
---

## Overview

Together with the [validating light node](../cross-chain-communication/index.md) the rollup circuits must ensure that incoming blocks are valid, that state is progressed correctly and that anyone can rebuild the state.

To support this, we construct a single proof for the entire block, which is then verified by the validating light node. This single proof is constructed by recursively merging proofs together in a binary tree structure. This structure allows us to keep the workload of each individual proof small, while making it very parallelizable. This works very well for the case where we want many actors to be able to participate in the proof generation.

The tree structure is outlined below, but the general idea is that we have a tree where all the leaves are transactions (kernel proofs) and through $\log(n)$ steps we can then "compress" them down to just a single root proof. Note that we have three (3) different types of "merger" circuits, namely:

- The base rollup
  - Merges two kernel proofs
- The merge rollup
  - Merges two base rollup proofs OR two merge rollup proofs
- The root rollup
  - Merges two merge rollup proofs

In the diagram the size of the tree is limited for show, but a larger tree will have more layers of merge rollups proofs. Circles mark the different types of proofs, while squares mark the different circuit types.

```mermaid
graph BT
    R_p((Root))
    R_c[Root]

    R_c --> R_p

    M0_p((Merge 0))
    M1_p((Merge 1))
    M0_p --> R_c
    M1_p --> R_c

    M0_c[Merge 0]
    M1_c[Merge 1]
    M0_c --> M0_p
    M1_c --> M1_p

    B0_p((Base 0))
    B1_p((Base 1))
    B2_p((Base 2))
    B3_p((Base 3))
    B0_p --> M0_c
    B1_p --> M0_c
    B2_p --> M1_c
    B3_p --> M1_c


    B0_c[Base 0]
    B1_c[Base 1]
    B2_c[Base 2]
    B3_c[Base 3]
    B0_c --> B0_p
    B1_c --> B1_p
    B2_c --> B2_p
    B3_c --> B3_p

    K0((Kernel 0))
    K1((Kernel 1))
    K2((Kernel 2))
    K3((Kernel 3))
    K4((Kernel 4))
    K5((Kernel 5))
    K6((Kernel 6))
    K7((Kernel 7))
    K0 --> B0_c
    K1 --> B0_c
    K2 --> B1_c
    K3 --> B1_c
    K4 --> B2_c
    K5 --> B2_c
    K6 --> B3_c
    K7 --> B3_c

    style R_p fill:#1976D2;
    style M0_p fill:#1976D2;
    style M1_p fill:#1976D2;
    style B0_p fill:#1976D2;
    style B1_p fill:#1976D2;
    style B2_p fill:#1976D2;
    style B3_p fill:#1976D2;
    style K0 fill:#1976D2;
    style K1 fill:#1976D2;
    style K2 fill:#1976D2;
    style K3 fill:#1976D2;
    style K4 fill:#1976D2;
    style K5 fill:#1976D2;
    style K6 fill:#1976D2;
    style K7 fill:#1976D2;
```

To understand what the circuits are doing and what checks they need to apply it is useful to understand what data is going into the circuits and what data is coming out.

Below is a figure of the data structures thrown around for the block proof creation. Note that the diagram does not include much of the operations for kernels, but mainly the data structures that are used for the rollup circuits.

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
}

class Header {
    last_archive: Snapshot
    body_hash: Fr[2]
    state: StateReference
    global_variables: GlobalVariables
}
Header *.. Body : body_hash
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
    l1_to_l2_messages: List~Fr~
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

class NewContractData {
    function_tree_root: Fr
    address: Address
    portal: EthAddress
}

class CombinedAccumulatedData {
    aggregation_object: AggregationObject
    read_requests: List~Fr~
    pending_read_requests: List~Fr~
    note_hashes: List~Fr~
    nullifiers: List~Fr~
    nullified_note_hashes: List~Fr~

    l2_to_l1_messages: List~Fr~
    contracts: List~NewContractData~
    public_update_requests: List~PublicDataUpdateRequest~
    public_reads: List~PublicDataRead~
    logs: Logs

    private_call_stack: List~CallRequest~
    public_call_stack: List~CallRequest~
    start_public_data_root: Fr
    end_public_data_root: Fr
}
CombinedAccumulatedData *-- "m" NewContractData: contracts
CombinedAccumulatedData *-- "m" PublicDataUpdateRequest: public_update_requests
CombinedAccumulatedData *-- "m" PublicDataRead: public_reads
CombinedAccumulatedData *-- Logs : logs

class ContractDeploymentData {
    deployer_public_key: Point
    constructor_vk_hash: Fr
    constructor_args_hash: Fr
    function_tree_root: Fr
    salt: Fr
    portal_address: Fr
}

class TxContext {
    fee_context: FeeContext
    is_contract_deployment: bool
    chain_id: Fr
    version: Fr
    contract_deployment_data: ContractDeploymentData
}
TxContext *-- ContractDeploymentData: contract_deployment_data

class CombinedConstantData {
    historical_header: Header
    tx_context: TxContext
}
CombinedConstantData *-- Header : historical_header
CombinedConstantData *-- TxContext : tx_context

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
  historical_header_membership_witnesses: List~HeaderMembershipWitness~
  kernel_data: List~KernelData~
  partial: PartialStateReference
  state_diff_hints: StateDiffHints
}
BaseRollupInputs *-- "m" KernelData : kernelData
BaseRollupInputs *-- PartialStateReference : partial
BaseRollupInputs *-- StateDiffHints : state_diff_hints
BaseRollupInputs *-- ConstantRollupData : constants

class BaseOrMergeRollupPublicInputs {
    type: Fr
    height_in_block_tree: Fr
    aggregation_object: AggregationObject
    txs_hash: Fr[2]
    out_hash: Fr[2]
    constants: ConstantRollupData
    start: PartialStateReference
    end: PartialStateReference
}
BaseOrMergeRollupPublicInputs *-- ConstantRollupData : constants
BaseOrMergeRollupPublicInputs *-- PartialStateReference : start
BaseOrMergeRollupPublicInputs *-- PartialStateReference : end

class ChildRollupData {
    proof: Proof
    public_inputs: BaseOrMergeRollupPublicInputs
}
ChildRollupData *-- BaseOrMergeRollupPublicInputs: public_inputs

class MergeRollupInputs {
    left: ChildRollupData
    right: ChildRollupData
}
MergeRollupInputs *-- ChildRollupData: left
MergeRollupInputs *-- ChildRollupData: right


class RootRollupInputs {
    l1_to_l2_msgs: List~Fr~
    l1_to_l2_msgs_sibling_path: List~Fr~
    parent: Header,
    parent_sibling_path: List~Fr~
    archive_sibling_path: List~Fr~
    left: ChildRollupData
    right: ChildRollupData
}
RootRollupInputs *-- ChildRollupData: left
RootRollupInputs *-- ChildRollupData: right
RootRollupInputs *-- Header : parent


class RootRollupPublicInputs {
    aggregation_object: AggregationObject
    archive: Snapshot
    header: Header
}
RootRollupPublicInputs *--Header : header
```

:::info CombinedAccumulatedData
Note that the `CombinedAccumulatedData` contains elements that we won't be using throughout the rollup circuits. However, as the data is used for the kernel proofs (when it is build recursively), we will include it here anyway.
:::

:::warning TODO  
Reconsider `ContractDeploymentData` in light of the newer (still being finalised) contract deployment flow  
:::

Since the diagram can be quite overwhelming, we will go through the different data structures and what they are used for along with the three (3) different rollup circuits.

### Higher-level tasks

Before looking at the circuits individually, it can however be a good idea to recall the reason we had them in the first place. For this, we are especially interested in the tasks that span multiple circuits and proofs.

#### State consistency

While the individual kernels are validated on their own, they might rely on state changes earlier in the block. For the block to be correctly validated, this means that when validating kernel $n$, it must be executed on top of the state after all kernels $<n$ have been applied. For example, when kernel $3$ is executed, it must be executed on top of the state after kernels $0$, $1$ and $2$ have been applied. If this is not the case, the kernel proof might be valid, but the state changes invalid which could lead to double spends.

It is therefore of the highest importance that the circuits ensure that the state is progressed correctly across circuit types and proofs. Logically, taking a few of the kernels from the above should be executed/proven as shown below, $k_n$ applied on top of the state that applied $k_{n-1}$

```mermaid
graph LR
    SM[State Machine]
    S0((State n-1))
    K0((Kernel n-1))
    S1((State n))

    S0 --> SM
    K0 --> SM
    SM --> S1


    SM_2[State Machine]
    K1((Kernel n))
    S2((State n+1))

    S1 --> SM_2
    K1 --> SM_2
    SM_2 --> S2

    style K0 fill:#1976D2;
    style K1 fill:#1976D2;
```

#### State availability

To ensure that state is made available, we could broadcast all of a block's input data as public inputs of the final root rollup proof, but a proof with so many public inputs would be very expensive to verify onchain. Instead we reduce the proof's public inputs by committing to the block's body by iteratively computing a `TxsHash` and `OutHash` at each rollup circuit iteration. AT the final iteration a `body_hash` is computed committing to the complete body.

To check that this body is published an Aztec node can reconstruct the `body_hash` from available data. Since we define finality as the point where the block is validated and included in the state of the [validating light node](../cross-chain-communication/index.md), we can define a block as being "available" if the validating light node can reconstruct the commitment `body_hash`.

Since we strive to minimize the compute requirements to prove blocks, we amortize the commitment cost across the full tree. We can do so by building merkle trees of partial "commitments", whose roots are ultimately computed in the final root rollup circuit. The `body_hash` is then computed from the roots of these trees, together with incoming messages.
Below, we outline the `TxsHash` merkle tree that is based on the `TxEffect`s and a `OutHash` which is based on the `l2_to_l1_msgs` (cross-chain messages) for each transaction. While the `TxsHash` implicitly includes the `l2_to_l1_msgs` we construct it separately since the `l2_to_l1_msgs` must be available to the L1 contract directly and not just proven available. This is not a concern when using L1 calldata as the data layer, but is a concern when using alternative data layers such as [Celestia](https://celestia.org/) or [Blobs](https://eips.ethereum.org/EIPS/eip-4844).

```mermaid
graph BT
    R[TxsHash]
    M0[Hash 0-1]
    M1[Hash 2-3]
    B0[Hash 0.0-0.1]
    B1[Hash 1.0-1.1]
    B2[Hash 2.0-2.1]
    B3[Hash 3.0-3.1]
    K0[TxEffect 0.0]
    K1[TxEffect 0.1]
    K2[TxEffect 1.0]
    K3[TxEffect 1.1]
    K4[TxEffect 2.0]
    K5[TxEffect 2.1]
    K6[TxEffect 3.0]
    K7[TxEffect 3.1]

    M0 --> R
    M1 --> R
    B0 --> M0
    B1 --> M0
    B2 --> M1
    B3 --> M1
    K0 --> B0
    K1 --> B0
    K2 --> B1
    K3 --> B1
    K4 --> B2
    K5 --> B2
    K6 --> B3
    K7 --> B3
```

```mermaid
graph BT
    R[OutHash]
    M0[Hash 0-1]
    M1[Hash 2-3]
    B0[Hash 0.0-0.1]
    B1[Hash 1.0-1.1]
    B2[Hash 2.0-2.1]
    B3[Hash 3.0-3.1]
    K0[l2_to_l1_msgs 0.0]
    K1[l2_to_l1_msgs 0.1]
    K2[l2_to_l1_msgs 1.0]
    K3[l2_to_l1_msgs 1.1]
    K4[l2_to_l1_msgs 2.0]
    K5[l2_to_l1_msgs 2.1]
    K6[l2_to_l1_msgs 3.0]
    K7[l2_to_l1_msgs 3.1]

    M0 --> R
    M1 --> R
    B0 --> M0
    B1 --> M0
    B2 --> M1
    B3 --> M1
    K0 --> B0
    K1 --> B0
    K2 --> B1
    K3 --> B1
    K4 --> B2
    K5 --> B2
    K6 --> B3
    K7 --> B3
```

The roots of these trees, together with incoming messages, makes up the `body_hash`.

```mermaid
graph BT
    R[body_hash]
    M0[TxsHash]
    M1[OutHash]
    M2[InHash]

    M3[l1_to_l2_messages]

    M0 --> R
    M1 --> R
    M2 --> R
    M3 --> M2
```

```python
def body_hash(body: Body):
    txs_hash = merkle_tree(body.txs, SHA256).root
    out_hash = merkle_tree([tx.l1_to_l2_msgs for tx in body.txs], SHA256).root
    in_hash = SHA256(body.l1_to_l2_messages)
    return SHA256(txs_hash, out_hash, in_hash)
```

:::info SHA256
SHA256 is used since as the hash function since it will likely be reconstructed outside the circuit in a resource constrained environment (Ethereum L1).
:::

## Next Steps

import DocCardList from '@theme/DocCardList';

<DocCardList />
