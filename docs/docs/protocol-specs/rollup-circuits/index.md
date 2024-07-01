---
title: Rollup Circuits
---

## Overview

Together with the [validating light node](../l1-smart-contracts/index.md), the rollup circuits must ensure that incoming blocks are valid, that state is progressed correctly, and that anyone can rebuild the state.

To support this, we construct a single proof for the entire block, which is then verified by the validating light node.
This single proof consist of three main components:
It has **two** sub-trees for transactions, and **one** tree for L1 to L2 messages.
The two transaction trees are then merged into a single proof and combined with the roots of the message tree to form the final proof and output.
Each of these trees are built by recursively combining proofs from a lower level of the tree.
This structure allows us to keep the workload of each individual proof small, while making it very parallelizable.
This works very well for the case where we want many actors to be able to participate in the proof generation.

Note that we have two different types of "merger" circuits, depending on what they are combining.

For transactions we have:

- The `merge` rollup
  - Merges two rollup proofs of either `base` or `merge` and constructs outputs for further proving
- The `root` rollup
  - Merges two rollup proofs of either `base` or `merge` and constructs outputs for L1

And for the message parity we have:

- The `root_parity` circuit
  - Merges `N` `root` or `base_parity` proofs
- The `base_parity` circuit
  - Merges `N` l1 to l2 messages in a subtree

In the diagram the size of the tree is limited for demonstration purposes, but a larger tree would have more layers of merge rollups proofs. Exactly how many layers and what combination of `base` and/or `merge` circuits are consumed is based on filling a [wonky tree](../state/tree-implementations.md#wonky-merkle-trees) with N transactions.
Circles mark the different types of proofs, while squares mark the different circuit types.

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
    K0 --> B0_c
    K1 --> B1_c
    K2 --> B2_c
    K3 --> B3_c

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

    R --> R_c

    R((RootParity))

    T0[BaseParity]
    T1[BaseParity]
    T2[BaseParity]
    T3[BaseParity]

    T0_P((RootParity 0))
    T1_P((RootParity 1))
    T2_P((RootParity 2))
    T3_P((RootParity 3))

    T4[RootParity]

    I0 --> T0
    I1 --> T1
    I2 --> T2
    I3 --> T3

    T0 --> T0_P
    T1 --> T1_P
    T2 --> T2_P
    T3 --> T3_P

    T0_P --> T4
    T1_P --> T4
    T2_P --> T4
    T3_P --> T4

    T4 --> R

    I0((MSG 0-3))
    I1((MSG 4-7))
    I2((MSG 8-11))
    I3((MSG 12-15))

    style R fill:#1976D2;
    style T0_P fill:#1976D2;
    style T1_P fill:#1976D2;
    style T2_P fill:#1976D2;
    style T3_P fill:#1976D2;
    style I0 fill:#1976D2;
    style I1 fill:#1976D2;
    style I2 fill:#1976D2;
    style I3 fill:#1976D2;
```

To understand what the circuits are doing and what checks they need to apply it is useful to understand what data is going into the circuits and what data is coming out.

Below is a figure of the data structures thrown around for the block proof creation.
Note that the diagram does not include much of the operations for kernels, but mainly the data structures that are used for the rollup circuits.

<!-- Missing `FeeContext` class definition in the diagram below -->
<!-- Missing `BaseRollupInputs` class definition in the diagram below  (Lasse: This is already in the diagram?) -->
<!-- Perhaps `KernelPublicInputs` needs to be renamed to specify which kernel circuits these public inputs apply to. Is it all public kernel circuits, for example? -->

<!-- NOTE: If you're editing this diagram, there will be other diagrams (e.g. in the state / circuits sections) that will need to be updated too. There are also class definitions in other sections which will need to be updated. -->

```mermaid
classDiagram
direction TB

class PartialStateReference {
    note_hash_tree: Snapshot
    nullifier_tree: Snapshot
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
    public_writes: List~PublicDataWrite~
    logs: Logs
}
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

class BaseParityInputs {
    msgs: List~Fr[2]~
}

class ParityPublicInputs {
    sha_root: Fr[2]
    converted_root: Fr
}

class RootParityInputs {
    children: List~ParityPublicInputs~
}
RootParityInputs *-- ParityPublicInputs: children

class RootParityInput {
    proof: Proof
    public_inputs: ParityPublicInputs
}
RootParityInput *-- ParityPublicInputs: public_inputs

class RootRollupInputs {
    l1_to_l2_roots: RootParityInput
    l1_to_l2_msgs_sibling_path: List~Fr~
    parent: Header,
    parent_sibling_path: List~Fr~
    archive_sibling_path: List~Fr~
    left: ChildRollupData
    right: ChildRollupData
}
RootRollupInputs *-- RootParityInput: l1_to_l2_roots
RootRollupInputs *-- ChildRollupData: left
RootRollupInputs *-- ChildRollupData: right
RootRollupInputs *-- Header : parent

class RootRollupPublicInputs {
    archive: Snapshot
    header: Header
}
RootRollupPublicInputs *--Header : header
```

:::info CombinedAccumulatedData
Note that the `CombinedAccumulatedData` contains elements that we won't be using throughout the rollup circuits.
However, as the data is used for the kernel proofs (when it is build recursively), we will include it here anyway.
:::

Since the diagram can be quite overwhelming, we will go through the different data structures and what they are used for along with the three (3) different rollup circuits.

### Higher-level tasks

Before looking at the circuits individually, it can however be a good idea to recall the reason we had them in the first place.
For this, we are especially interested in the tasks that span multiple circuits and proofs.

#### State consistency

While the individual kernels are validated on their own, they might rely on state changes earlier in the block.
For the block to be correctly validated, this means that when validating kernel $n$, it must be executed on top of the state after all kernels $<n$ have been applied.
For example, when kernel $3$ is executed, it must be executed on top of the state after kernels $0$, $1$ and $2$ have been applied.
If this is not the case, the kernel proof might be valid, but the state changes invalid which could lead to double spends.

It is therefore of the highest importance that the circuits ensure that the state is progressed correctly across circuit types and proofs.
Logically, taking a few of the kernels from the above should be executed/proven as shown below, $k_n$ applied on top of the state that applied $k_{n-1}$

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

To ensure that state is made available, we could broadcast all of a block's input data as public inputs of the final root rollup proof, but a proof with so many public inputs would be very expensive to verify onchain.

Instead, we can reduce the number of public inputs by committing to the block's body and iteratively "build" up the commitment at each rollup circuit iteration.
At the very end, we will have a commitment to the transactions that were included in the block (`TxsHash`), the messages that were sent from L2 to L1 (`OutHash`) and the messages that were sent from L1 to L2 (`InHash`).

To check that the body is published an Aztec node can simply reconstruct the hashes from available data.
Since we define finality as the point where the block is validated and included in the state of the [validating light node](../l1-smart-contracts/index.md), we can define a block as being "available" if the validating light node can reconstruct the commitment hashes.

Since the `InHash` is directly computed by the `Inbox` contract on L1, the data is obviously available to the contract without doing any more work.
Furthermore, the `OutHash` is a computed from a subset of the data in `TxsHash` so if it is possible to reconstruct `TxsHash` it is also possible to reconstruct `OutHash`.

Since we strive to minimize the compute requirements to prove blocks, we amortize the commitment cost across the full tree.
We can do so by building merkle trees of partial "commitments", whose roots are ultimately computed in the final root rollup circuit.
Below, we outline the `TxsHash` merkle tree that is based on the `TxEffect`s and a `OutHash` which is based on the `l2_to_l1_msgs` (cross-chain messages) for each transaction, with four transactions in this rollup.
While the `TxsHash` implicitly includes the `OutHash` we need it separately such that it can be passed to the `Outbox` for consumption by the portals with minimal work.

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
    B0[Hash 0.0-0.3]
    B1[Hash 1.0-1.1]
    B2[Hash 2.0-2.1]
    B3[Hash 3.0-3.3]
    K0[l2_to_l1_msgs 0.0-0.1]
    K1[l2_to_l1_msgs 0.2-0.3]
    K2[l2_to_l1_msgs 1.0]
    K3[l2_to_l1_msgs 1.1]
    K4[l2_to_l1_msgs 2.0]
    K5[l2_to_l1_msgs 2.1]
    K6[l2_to_l1_msgs 3.0-3.1]
    K7[l2_to_l1_msgs 3.2-3.3]
    K8[l2_to_l1_msgs 0.0]
    K9[l2_to_l1_msgs 0.1]
    K10[l2_to_l1_msgs 0.2]
    K11[l2_to_l1_msgs 0.3]
    K12[l2_to_l1_msgs 3.0]
    K13[l2_to_l1_msgs 3.1]
    K14[l2_to_l1_msgs 3.2]
    K15[l2_to_l1_msgs 3.3]

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
    K8 --> K0
    K9 --> K0
    K10 --> K1
    K11 --> K1
    K12 --> K6
    K13 --> K6
    K14 --> K7
    K15 --> K7
```

```mermaid
graph BT
    R[InHash]
    M0[Hash 0-1]
    M1[Hash 2-3]
    B0[Hash 0.0-0.1]
    B1[Hash 1.0-1.1]
    B2[Hash 2.0-2.1]
    B3[Hash 3.0-3.1]
    K0[l1_to_l2_msgs 0.0]
    K1[l1_to_l2_msgs 0.1]
    K2[l1_to_l2_msgs 1.0]
    K3[l1_to_l2_msgs 1.1]
    K4[l1_to_l2_msgs 2.0]
    K5[l1_to_l2_msgs 2.1]
    K6[l1_to_l2_msgs 3.0]
    K7[l1_to_l2_msgs 3.1]

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

While the `TxsHash` merely require the data to be published and known to L1, the `InHash` and `OutHash` needs to be computable on L1 as well.
This reason require them to be efficiently computable on L1 while still being non-horrible inside a snark - leading us to rely on SHA256.


The L2 to L1 messages from each transaction form a variable height tree. In the diagram above, transactions 0 and 3 have four messages, so require a tree with two layers, whereas the others only have two messages and so require a single layer tree. The base rollup calculates the root of this tree and passes it as the to the next layer. Merge rollups simply hash both of these roots together and pass it up as the `OutHash`.

## Next Steps

import DocCardList from '@theme/DocCardList';

<DocCardList />
