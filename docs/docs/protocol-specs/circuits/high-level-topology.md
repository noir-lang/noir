# High Level Topology

## Overview

A transaction begins with a call to a private function, which may invoke nested calls to other private and public functions. The entire set of private function calls is executed in a secure environment, and their proofs are validated and aggregated by private kernel circuits. Meanwhile, any public function calls triggered from private functions will be enqueued. The proofs for these calls, along with those from the nested public function calls, are generated and processed through public kernel circuits in any entity possessing the correct contexts.

Once all functions in a transaction are executed, the accumulated data is outputted from a tail circuit. These values are then inserted or updated to the [state trees](../state/index.md) within the base rollup circuit. The merge rollup circuit facilitates the merging of two rollup proofs. Repeating this merging process enables the inclusion of more transactions in a block. Finally, the root rollup circuit produces the final proof, which is subsequently submitted and validated onchain.

To illustrate, consider a transaction involving the following functions, where circles depict private functions, and squares denote public functions:

:::info
A note for Aztec protocol developers: In this protocol spec, the order in which the kernel circuit processes calls is different from previous literature, and is different from the current implementation (as at January 2024).
:::

<!-- Mike review: perhaps a more comprehensive example would be if f2 makes the calls to f4 and f5, to cover a case which isn't covered in the current example: If f2 calls f4 & f5, then which is processed by the kernel first out of f3, f4, or f5?
-->

```mermaid
flowchart LR
    f0([f0]) --> f1([f1])
    f0 --> f2([f2])
    f0 --> f3([f3])
    f1 -.-> F0
    F0 --> F1
    F0 --> F2
    F2 --> F3
    f3 --> f4([f4])
    f3 -.-> F4
    f3 --> f5([f5])
```

This transaction contains 6 private functions (f0 to f5) and 5 public functions (F0 to F4), with `f0` being the entrypoint. The entire transaction is processed as follows:

```mermaid
flowchart TB
    subgraph Transaction A
        subgraph Private Functions
            f0([f0])
            f1([f1])
            f2([f2])
            f3([f3])
            f4([f4])
            f5([f5])
        end
        subgraph Public Functions
            F0
            F1
            F2
            F3
            F4
        end
    end
    subgraph Transaction C
      init2(...)
      tail2(Tail Private Kernel)
      init2 -.-> tail2
    end
    subgraph Transaction B
      init1(...)
      tail1(Tail Private Kernel)
      init1 -.-> tail1
    end
    subgraph Public Kernel
        INIT0(Initial Public Kernel)
        INNER0(Inner Public Kernel)
        INNER1(Inner Public Kernel)
        INNER2(Inner Public Kernel)
        INNER3(Inner Public Kernel)
        TAIL0(Tail Public Kernel)
        INIT0 --> INNER0
        INNER0 --> INNER1
        INNER1 --> INNER2
        INNER2 --> INNER3
        INNER3 --> TAIL0
    end
    subgraph Private Kernel
        init0(Initial Private Kernel)
        inner0(Inner Private Kernel)
        inner1(Inner Private Kernel)
        inner2(Inner Private Kernel)
        reset0(Reset Private Kernel)
        inner3(Inner Private Kernel)
        inner4(Inner Private Kernel)
        reset1(Reset Private Kernel)
        tail0(Tail Private Kernel)
        init0 --> inner0
        inner0 --> inner1
        inner1 --> inner2
        inner2 --> reset0
        reset0 --> inner3
        inner3 --> inner4
        inner4 --> reset1
        reset1 --> tail0
    end
    f0 --> init0
    f1 --> inner0
    f2 --> inner1
    f3 --> inner2
    f4 --> inner3
    f5 --> inner4
    F0 --> INIT0
    F1 --> INNER0
    F2 --> INNER1
    F3 --> INNER2
    F4 --> INNER3
    subgraph Rollup
        BR0(Base Rollup)
        BR1(Base Rollup)
        BR2(Base Rollup)
        BR3(Base Rollup)
        MR0(Merge Rollup)
        MR1(Merge Rollup)
        MR2(Merge Rollup)
        MR3(Merge Rollup)
        ROOT(Root Rollup)
    end
    tail0 --> INIT0
    TAIL0 --> BR0
    tail1 --> BR1
    tail2 --> BR2
    BR0 --> MR0
    BR1 --> MR0
    BR2 --> MR1
    BR3 --> MR1
    MR0 --> MR2
    MR1 --> MR2
    MR2 --> ROOT
    MR3 --> ROOT
```

A few things to note:

- A transaction always starts with an [initial private kernel circuit](./private-kernel-initial.mdx).
- An [inner private kernel circuit](./private-kernel-inner.mdx) won't be required if there is only one private function in a transaction.
- A [reset private kernel circuit](./private-kernel-reset.md) can be executed between two private kernel circuits to "reset" transient data. The reset process can be repeated as needed.
- Public functions are "enqueued" when invoked from a private function. Public kernel circuits will be executed after the completion of all private kernel iterations.
- A [base rollup circuit](../rollup-circuits/base-rollup.md) can accept either a [tail public kernel circuit](./public-kernel-tail.md), or a [tail private kernel circuit](./private-kernel-tail.md) in cases where no public functions are present in the transaction.
- A [merge rollup circuit](../rollup-circuits/merge-rollup.md) can merge two base rollup circuits or two merge rollup circuits.
- The final step is the execution of the [root rollup circuit](../rollup-circuits/root-rollup.md), which combines two base rollup circuits or two merge rollup circuits.
