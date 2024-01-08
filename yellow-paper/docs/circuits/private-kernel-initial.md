# Private Kernel Circuit - Initial

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

In the **initial** kernel iteration, the process involves taking a transaction request and private call data, verifying their integrity, and preparing the necessary data for subsequent circuits to operate. This step is particularly beneficial due to its separation from the [inner private kernel circuit](./private-kernel-inner.md), as the first call lacks a "previous kernel" to process. Additionally, it executes tasks that are pertinent to a transaction and need only occur once.

### Key Responsibilities Specific to this Circuit

#### Validating the correspondence of function call with caller's intent.

This entails ensuring that the following data from the private call aligns with the specifications in the transaction request:

- Contract address.
- [Function data](#function_data).
- Function arguments.

> Although it's not enforced in the protocol, it is customary to provide a signature signed over the transaction request and verify it in the first function call. This practice guarantees that only the party possessing the key(s) can authorize a transaction with the exact transaction request.

#### Verifying the legitimacy of the function as the entrypoint.

- It must be a private function.
- It must not be an internal function.

#### Ensuring the function call is the first call.

- It must not be a delegate call.
- It must not be a static call.

#### Ensuring transaction uniqueness.

- It must emit the hash of the transaction request as the **first** nullifier.

This nullifier serves multiple purposes:

- Identifying a transaction.
- Preventing the signature of a transaction request from being reused in another transaction.
- Generating values that should be maintained within the transaction's scope. For example, it is utilized to compute the nonces for all the note hashes in a transaction.

> Note that the final transaction data is not deterministic for a given transaction request. The production of new notes, the destruction of notes, and various other values are likely to change based on the time and conditions when a transaction is being composed. However, the intricacies of implementation should not be a concern for the entity initiating the transaction.

### Processing Private Function Call

#### Ensuring the contract instance being called is deployed.

It proves that the nullifier representing the contract exists in the contract tree.

This nullifier is the contract address siloed with the address of a precompiled deployment contract.

#### Ensuring the function being called exists in the contract.

The contract address contains the contract class ID, which is a hash of the root of its function tree and additional values. This circuit leverages these characteristics to establish the validity of the function's association with the contract address.

Each leaf of the function tree is a hash representing a function. The preimage includes:

- Function data.
- Hash of the verification key.
- Hash of the function bytecode.

To ensure the function's existence, the circuit executes the following steps:

1. Computes the hash of the verification key.
2. Calculates the function leaf: `hash(...function_data, vk_hash, bytecode_hash)`
3. Derives the function tree root with the leaf and the specified sibling path.
4. Computes the contract class ID using the function tree root and additional information.
5. Generates the contract address using the contract class ID and other relevant details.
6. Validates that the contract address matches the address specified in the private call data.

#### Verifying the private function proof.

It verifies that the private function was executed successfully with the provided proof data, verification key, and the public inputs of the private function circuit.

#### Verifying the public inputs of the private function circuit.

It ensures the private function circuit's intention by checking the following:

- The contract address for each non-empty item in the following arrays must equal the current contract address:
  - Note hash contexts.
  - Nullifier contexts.
  - L2-to-L1 message contexts.
  - Read requests.
- The portal contract address for each non-empty L2-to-L1 message context must equal the current portal contract address.
- If the new contract contexts array is not empty, the contract address must equal the precompiled deployment contract address.
- The historical data must match the one in the constant data.

> Ensuring the alignment of the contract addresses is crucial, as it is later used to silo the value and to establish associations with values within the same contract.

#### Verifying the call requests.

For both private and public call requests initiated in the current function call, it ensures that for each request at index _i_:

- Its hash equals the value at index _i_ within the call request hashes array in private function circuit's public inputs.
- Its caller context is either empty or aligns with the call context of the current function call, including:
  - _msg_sender_
  - Storage contract address.

> It is important to note that the caller context in a call request may be empty for standard calls. This precaution is crucial to prevent information leakage, particularly as revealing the _msg_sender_ to the public could pose security risks when calling a public function.

#### Verifying the counters.

It verifies that each relevant value is associated with a legitimate counter.

1. For the current call:

   - The _counter_start_ must be 0.
   - The _counter_end_ must be greater than the _counter_start_.

2. For private call requests:

   - The _counter_end_ of each request must be greater than its _counter_start_.
   - The _counter_start_ of the first request must be greater than the _counter_start_ of the current call.
   - The _counter_start_ of the second and subsequent requests must be greater than the _counter_end_ of the previous request.
   - The _counter_end_ of the last request must be less than the _counter_end_ of the current call.

3. For items in each ordered array created in the current call:

   - The counter of the first item much be greater than the _counter_start_ of the current call.
   - The counter of each subsequent item much be greater than the counter of the previous item.
   - The counter of the last item much be less than the _counter_end_ of the current call.

   The ordered arrays include:

   - Note hash contexts.
   - Nullifier contexts.
   - New contract contexts.
   - Read requests.
   - Public call requests.

   > Note that _counter_start_ is used in the above steps for public call requests to ensure their correct ordering. At this point, the _counter_end_ of public call request is unknown. Both counters will be [recalibrated](./private-kernel-tail.md#recalibrating-counters) in the tail circuit following the simulation of all public function calls.

### Validating Public Inputs

#### Verifying the accumulated data.

It verifies that the following values align with those in the private call data:

- Log hashes.
- Log lengths.

#### Verifying the transient accumulated data.

1. It ensures that the following arrays match those in the private call data:

   - Note hash contexts.
   - Nullifier contexts.
   - L2-to-L1 message contexts.
   - New contract contexts.
   - Read requests.
   - Public call requests.

2. It checks that the following aligns with the array in the private call data, with items arranged in **reverse** order:

   - Private call requests.

   > It's important that the call requests are arranged in reverse order to ensure they are executed in chronological order. This becomes particularly crucial when calling a contract deployed earlier within the same transaction.

3. For the note hash contexts, it also verifies that each is associated with a nullifier counter, which is provided as a hint via the private inputs. The nullifier counter can be:

   - Zero: if the note is not nullified in the same transaction.
   - Greater than zero: if the note is nullified in the same transaction.
     - This value must be greater than the counter of the note hash.

   > Nullifier counters are used in the [reset private kernel circuit](./private-kernel-reset.md#verifying-read-requests) to ensure a read happens **before** a transient note is nullified.

   > Zero can be used to indicate a non-existing transient nullifier, as this value can never serve as the counter of a nullifier. It corresponds to the _counter_start_ of the first function call.

#### Verifying the constant data.

It verifies that:

- The transaction context matches the one in the transaction request.

> The historical data must align with the data used in the private function circuit, as verified [earlier](#verifying-the-public-inputs-of-the-private-function-circuit).

## Private Inputs

### Transaction Request

A transaction request represents the caller's intent. It contains:

- Sender's address.
- <a name="function_data">Function data</a>:

  - Function selector.
  - Function type (private/public/unconstrained).
  - A flag indicating whether the function is an internal function.

- Hash of the function arguments.
- Transaction context
  - A flag indicating whether it is a fee paying transaction.
  - A flag indicating whether it is a fee rebate transaction.
  - Chain ID.
  - Version of the transaction.

### Private Call Data

The private call data holds details about the current private function call:

- Contract address.
- Function data.
- Private call requests.
- Public call requests.
- Private function circuit public inputs.
- Proof of the private function circuit.
- Verification key of the private function circuit.
- Hash of the function bytecode.

### Hints

Data that aids in the verifications carried out in this circuit or later iterations:

- Membership witness for the function leaf.
- Membership witness for the contract leaf.
- Transient note nullifier counters.

## Public Inputs

The structure of this public inputs aligns with that of the [inner private kernel circuit](./private-kernel-inner.md) and the [reset private kernel circuit](./private-kernel-reset.md).

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

It contains the result from the current function call:

- Log hashes.
- Log lengths.

### Transient Accumulated Data

It includes transient data accumulated during the execution of the transaction up to this point:

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.
- New contract contexts.
- Read requests.
- Private call requests.
- Public call requests.
