# Private Kernel Circuit - Tail

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

The **tail** circuit abstains from processing individual private function calls. Instead, it incorporates the outcomes of a private kernel circuit and conducts additional processing essential for generating the final public inputs suitable for submission to the transaction pool, subsequently undergoing processing by Sequencers and Provers. The final public inputs must safeguard against revealing any private information unnecessary for the execution of public kernel circuits and rollup circuits.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs.

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.md).
- [Inner private kernel proof](./private-kernel-inner.md).
- [Reset private kernel proof](./private-kernel-reset.md).

An inner iteration may be omitted when there's only a single private function call for the transaction. And a reset iteration can be skipped if there are no read requests and transient nullifiers in the public inputs from the last initial or inner iteration.

#### Ensuring the previous iteration is the last.

The following must be empty to ensure all the private function calls are processed:

- Private call requests.

The following must be empty to ensure a comprehensive final reset:

- The nullified note hash associated with each nullifier.
- Read requests.

> A [reset iteration](./private-kernel-reset.md) should ideally precede this step. Although it doesn't have to be executed immediately before the tail circuit, as long as it effectively clears the specified values.

### Processing Final Outputs

#### Siloing values.

1. This circuit must silo the following with each item's contract address:

   - Note hash contexts.
   - Nullifier contexts.

   The siloed value is computed as: `hash(contract_address, value)`.

   Siloing with a contract address ensures that data produced by a contract is accurately attributed to the correct contract and cannot be misconstrued as data created in a different contract.

2. The circuit then applies nonces to the note hashes:

   - The nonce for a note hash is computed as: `hash(first_nullifier, index)`, where:
     - `first_nullifier` is the [hash of the transaction request](./private-kernel-initial.md#ensuring-transaction-uniqueness).
     - `index` is the position of the note hash in the note hashes array in the public inputs.

   Siloing with a nonce guarantees that each final note hash is a unique value in the note hash tree.

3. Additionally, this circuit generates the final hashes for L2-L1 messages, calculated as:

   `hash(contract_address, version_id, portal_contract_address, chain_id, message)`

   Where _version_id_ and _portal_contract_address_ equal the values defined in the constant data.

#### Verifying ordered arrays.

The initial and inner kernel iterations may produce values in an unordered state due to the serial nature of the kernel, contrasting with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following arrays (_ordered_array_) in public inputs:

- Note hashes.
- Nullifiers.
- L2-to-l1 messages.
- Public call requests.
- New contracts (if public call request is empty).
- New contract contexts (if public call request is not empty).

The corresponding _unordered_arrays_ for the above are sourced either from the transient accumulated data of the previous iteration or from the [siloed results](#siloing-values).

A hints array is provided through private inputs for every _unordered_array_.

For each hint _hints[i]_ at index _i_, this circuit locates the item at index _i_ in _ordered_array_:

- If the item is not empty:
  - It must correspond to the item at index _hints[i]_ in _unordered_array_.
  - For _i_ != 0, the counter must be greater (less for public call requests) than the counter of the item at index _hints[i - 1]_ in _unordered_array_.
- If the item is empty:
  - All the subsequent items must be empty in both _ordered_array_ and _unordered_array_.

> Note that the public call requests must be arranged in descending order to ensure the calls are executed in chronological order.

> Note that while ordering could occur gradually in each kernel iteration, the implementation is much simpler and **typically** more efficient to be done once in the tail circuit.

#### Recalibrating counters.

1. For public call requests:

   The _counter_end_ for a public call request is determined by the overall count of call requests, reads and writes, note hashes and nullifiers within its scope, including those nested within its child function executions. This calculation, performed in advance of executing this circuit, provides the necessary input for the recalibration process.

   This circuit enables the adjustment of counters, ensuring that subsequent public kernels can be executed with the correct counter range.

   An array _public_call_counters_ is provided through private inputs. The reassignment process unfolds as follows:

   1. Check that items in _public_call_counters_ are in descending order:
      - The _counter_end_ of each item must be greater than its _counter_start_.
      - The _counter_end_ of the second and subsequent items must be less than the _counter_start_ of the previous item.
   2. Ensure that the _counter_start_ of the last item in _public_call_counters_ is _1_.
   3. Assign the _counter_start_ and _counter_end_ of the item at index _i_ in _public_call_counters_ to the corresponding item at index _i_ in the [ordered](#verifying-ordered-arrays) public call requests.

   > It's crucial for the _counter_start_ of the last item to be _1_, as it's assumed in the [tail public kernel circuit](./public-kernel-tail.md#grouping-update-requests) that no update requests have a counter _1_.

   > While the _counter_start_ of public call request is assigned in the private function circuit to preserve the order, it's important to acknowledge that it may be modified in this step. As using _counter_start_ populated from private function circuits maybe leak information. It's recommended to adept the values that mirror the incremented amount on the public side without including any side effects on the private side.

2. For new contract contexts:

   If there's at least one non-empty public call request, the new contract contexts will be carried forward for processing in the public kernels. However, the counters in new contract contexts must be adjusted to reflect the changes to the counters for public call requests in the previous step.

   For each new contract context in the transient accumulated data:

   1. If its counter is greater than the **old** _counter_start_ of the public call request at index _0_, update it to be the **new** _counter_end_ of the public call request and skip the remaining steps.
   2. Find the public call request at index _i_ where the **old** _counter_start_ is greater than the counter.
   3. Check that the **old** _counter_start_ of the public call request at index _i + 1_ is less than the counter.
   4. Update its counter to be the **new** _counter_start_ of the public call request at index _i_.

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - Note hashes.
   - Nullifiers.
   - L2-to-L1 messages.
   - New contracts.

   > Note that these are arrays of siloed values or relevant data. Attributes aiding verification and siloing only exist in the corresponding types in the transient accumulated data.

2. The following must match the respective values in the previous kernel's public inputs:

   - Log hashes.
   - Log lengths.

3. The following must be empty:

   - Old public data tree snapshot.
   - New public data tree snapshot.

#### Verifying the transient accumulated data.

It ensures that all data in the transient accumulated data is empty, with the exception of the public call requests and new contract contexts.

The public call requests must adhere to a specific order, as verified in a [previous step](#verifying-ordered-arrays).

The new contract contexts should be empty when there are no public call requests. In the event of propagation to the public kernels, they must also [conform to a specific order](#verifying-ordered-arrays).

#### Verifying the constant data.

It verifies that the constant data matches the one in the previous iteration's public inputs.

## Private Inputs

### Previous Kernel

The data of the previous kernel iteration:

- Proof of the kernel circuit. It could be one of the following:
  - [Initial private kernel circuit](./private-kernel-initial.md).
  - [Inner private kernel circuit](./private-kernel-inner.md).
  - [Reset private kernel circuit](./private-kernel-reset.md).
- Public inputs of the proof.
- Verification key of the kernel circuit.
- Membership witness for the verification key.

### Hints

Data that aids in the verifications carried out in this circuit:

- Sorted indices of public call requests.
- Counters for public call requests.

## Public Inputs

The structure of this public inputs aligns with that of the [iterative public kernel circuit](./public-kernel-iterative.md) and the [tail public kernel circuit](./public-kernel-tail.md).

### Constant Data

These are constants that remain the same throughout the entire transaction:

- Historical data - representing the states of the block at which the transaction is constructed, including:
  - Hash of the global variables.
  - Roots of the trees:
    - Note hash tree.
    - Nullifier tree.
    - Contract tree.
    - L1-to-l2 message tree.
    - Public data tree.
- Transaction context
  - A flag indicating whether it is a fee paying transaction.
  - A flag indicating whether it is a fee rebate transaction.
  - Chain ID.
  - Version of the transaction.

### Accumulated Data

It contains data accumulated during the execution of the transaction:

- Note hashes.
- Nullifiers.
- L2-to-L1 messages.
- New contracts.
- Log hashes.
- Log lengths.
- Old public data tree snapshot.
- New public data tree snapshot.

### Transient Accumulated Data

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.
- New contract contexts.
- Read requests.
- Update requests.
- Public call requests.
