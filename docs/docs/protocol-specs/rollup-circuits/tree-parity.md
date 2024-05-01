---
title: L1 to L2 Message Parity
---

To support easy consumption of l1 to l2 messages inside the proofs, we need to convert the tree of messages to a snark-friendly format.

If you recall back in [L1 smart contracts](./../l1-smart-contracts/index.md#inbox) we were building a message tree on the L1.
We used SHA256 to compute the tree which is cheap to compute on L1.
As SHA256 is not snark-friendly, weak devices would not be able to prove inclusion of messages in the tree.

This circuit is responsible for converting the tree such that users can easily build the proofs.
We essentially use this circuit to front-load the work needed to prove the inclusion of messages in the tree.
As earlier we are using a tree-like structure.
Instead of having a `base`, `merge` and `root` circuits, we will have only `base` and `root` parity circuits.
We only need these two, since what would have been the `merge` is doing the same as the `root` for this case.

```mermaid
graph BT
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

The output of the "combined" circuit will be the `converted_root` which is the root of the snark-friendly message tree.
And the `sha_root` which must match the root of the sha256 message tree from the L1 Inbox.
The circuit computes the two trees using the same inputs, and then we ensure that the elements of the trees match the inbox later in the [state transitioner](./../l1-smart-contracts/index.md#overview).
It proves parity of the leaves in the two trees.

```mermaid
classDiagram
direction LR

class ParityPublicInputs {
    sha_root: Fr[2]
    converted_root: Fr
}

class RootParityInputs {
    children: List~RootParityInput~
}

RootParityInputs *-- RootParityInput: children

class RootParityInput {
    proof: Proof
    verification_key: VerificationKey
    public_inputs: ParityPublicInputs
}
RootParityInput *-- ParityPublicInputs: public_inputs

class BaseParityInputs {
    msgs: List~Fr[2]~
}
```

The logic of the circuits is quite simple - build both a SHA256 and a snark-friendly tree from the same inputs.
For optimization purposes, it can be useful to have the layers take more than 2 inputs to increase the task of every layer.
If each just take 2 inputs, the overhead of recursing through the layers might be higher than the actual work done.
Recall that all the inputs are already chosen by the L1, so we don't need to worry about which to chose.

```python
def base_parity_circuit(inputs: BaseParityInputs) -> ParityPublicInputs:
    sha_root = MERKLE_TREE(inputs.msgs, SHA256);
    converted_root = MERKLE_TREE(inputs.msgs, SNARK_FRIENDLY_HASH_FUNCTION);
    return ParityPublicInputs(sha_root, converted_root)

def root_parity_circuit(inputs: RootParityInputs) -> ParityPublicInputs:
    for msg in inputs.children:
        assert msg.proof.verify(msg.public_inputs, msg.verification_key);

    sha_root = MERKLE_TREE(
      [msg.public_inputs.sha_root for msg in inputs.children],
      SHA256
    );
    converted_root = MERKLE_TREE(
      [msg.public_inputs.converted_root for msg in inputs.children],
      SNARK_FRIENDLY_HASH_FUNCTION
    );
    return ParityPublicInputs(sha_root, converted_root)
```
