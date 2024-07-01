---
title: Merge Rollup
---

The Merge rollup circuit is our in-between circuit, it doesn't need to perform any state updates, but mainly check the consistency of its inputs.

```mermaid
graph LR
A[MergeRollupInputs] --> C[MergeRollupCircuit] --> B[BaseOrMergeRollupPublicInputs]
```

## Overview

Below is a subset of the data structures figure from earlier for easy reference.

```mermaid
classDiagram
direction TB

class PartialStateReference {
    note_hash_tree: Snapshot
    nullifier_tree: Snapshot
    contract_tree: Snapshot
    public_data_tree: Snapshot
}

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

class ConstantRollupData {
  last_archive: Snapshot
  base_rollup_vk_hash: Fr,
  merge_rollup_vk_hash: Fr,
  global_variables: GlobalVariables
}
ConstantRollupData *-- GlobalVariables : global_variables

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
```

### Validity Conditions

```python
def MergeRollupCircuit(
    left: ChildRollupData,
    right: ChildRollupData
) -> BaseOrMergeRollupPublicInputs:
    assert left.proof.is_valid(left.public_inputs)
    assert right.proof.is_valid(right.public_inputs)

    assert left.public_inputs.constants == right.public_inputs.constants
    assert left.public_inputs.end == right.public_inputs.start
    assert left.public_inputs.num_txs >= right.public_inputs.num_txs

    return BaseOrMergeRollupPublicInputs(
        type=1,
        num_txs=left.public_inputs.num_txs + right.public_inputs.num_txs,
        txs_effect_hash=SHA256(left.public_inputs.txs_effect_hash | right.public_inputs.txs_effect_hash),
        out_hash=SHA256(left.public_inputs.out_hash | right.public_inputs.out_hash),
        start=left.public_inputs.start,
        end=right.public_inputs.end,
        constants=left.public_inputs.constants
    )
```
