# Private Kernel Circuit - Inner

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

Each **inner** kernel iteration processes a private function call and the results of a previous kernel iteration.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs.

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.md)
- Inner private kernel proof.
- [Reset private kernel proof](./private-kernel-reset.md).

The previous proof and the proof for the current function call are verified using recursion.

### Processing Private Function Call

#### Ensuring the contract instance being called is deployed.

It proves that either of the following conditions is true:

1. The nullifier representing the contract exists in the contract tree.

   - Specifically, this nullifier is the contract address siloed with the address of a precompiled deployment contract.

2. The contract is listed in the new contract context within the public inputs.

   - The index of this contract in the new contracts array is supplied as a hint through private inputs.
   - The counter of the new contract context must be less than the _counter_start_ of the current call.

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

#### Ensuring the function is legitimate:

- It must be a private function.

#### Ensuring the current call matches the call request.

The top item in the previous iteration's private call requests must pertain to the current function call.

This circuit will pop the request from the stack, comparing the hash with that of the current function call.

The preimage of the hash contains:

- Contract address.
- Function data.
- Private function circuit's public inputs.

#### Ensuring this function is called with the correct context.

1. If it is a standard call:

   - The storage contract address of the current iteration must be the same as its contract address.
   - The _msg_sender_ of the current iteration must be the same as the caller's contract address.

2. If it is a delegate call:

   - The caller context in the call request must not be empty. Specifically, the following values of the caller should not be zeros:
     - _msg_sender_.
     - Storage contract address.
   - The _msg_sender_ of the current iteration must equal the caller's _msg_sender_.
   - The storage contract address of the current iteration must equal the caller's storage contract address.
   - The storage contract address of the current iteration must NOT equal the contract address.

3. If it is an internal call:

   - The _msg_sender_ of the current iteration must equal the storage contract address.

#### Verifying the private function proof.

It verifies that the private function was executed successfully with the provided proof data, verification key, and the public inputs of the private function circuit.

This circuit verifies this proof and [the proof for the previous function call](#verifying-the-previous-kernel-proof) using recursion, and generates a single proof. This consolidation of multiple proofs into one is what allows the private kernel circuits to gradually merge private function proofs into a single proof of execution that represents the entire private section of a transaction.

#### Verifying the public inputs of the private function circuit.

It ensures the private function circuit's intention by checking the following:

- The contract address for each non-empty item in the following arrays must equal the storage contract address of the current call:
  - Note hash contexts.
  - Nullifier contexts.
  - L2-to-L1 message contexts.
  - Read requests.
- The portal contract address for each non-empty L2-to-L1 message must equal the portal contract address of the current call.
- If the new contract contexts array is not empty, the contract address must equal the precompiled deployment contract address.
- The historical data must match the one in the constant data.

> Ensuring the alignment of the contract addresses is crucial, as it is later used to silo the value and to establish associations with values within the same contract.

If it is a static call, it must ensure that the function does not induce any state changes by verifying that the following arrays are empty:

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.

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

   - The _counter_end_ of the current call must be greater than its _counter_start_.
   - Both counters must match the ones defined in the top item in the previous iteration's private call requests.

2. For private call requests in the private call data:

   - The _counter_end_ of each request must be greater than its _counter_start_.
   - The _counter_start_ of the first request must be greater than the _counter_start_ of the current call.
   - The _counter_start_ of the second and subsequent requests must be greater than the _counter_end_ of the previous request.
   - The _counter_end_ of the last request must be less than the _counter_end_ of the current call.

3. For items in each ordered array in the private call data:

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

It checks that the hashes and the lengths for both encrypted and unencrypted logs are accumulated as follows:

- New log hash = `hash(prev_hash, cur_hash)`
  - If either hash is zero, the new hash will be `prev_hash | cur_hash`
- New log length = `prev_length + cur_length`

#### Verifying the transient accumulated data.

1. It verifies that the following values match the result of combining the values in the previous iteration's public inputs with those in the private call data:

   - Note hash contexts.
   - Nullifier contexts.
   - L2-to-L1 message contexts.
   - New contract contexts.
   - Read requests.
   - Public call requests.

2. For the newly added note hashes from private function circuits' public inputs, this circuit also checks that each is associated with a nullifier counter, provided as a hint via the private inputs. The nullifier counter can be:

   - Zero: if the note is not nullified in the same transaction.
   - Greater than zero: if the note is nullified in the same transaction.
     - This value must be greater than the counter of the note hash.

   > Nullifier counters are used in the [reset private kernel circuit](./private-kernel-reset.md) to ensure a read happens **before** a transient note is nullified.

   > Zero can be used to indicate a non-existing transient nullifier, as this value can never serve as the counter of a nullifier. It corresponds to the _counter_start_ of the first function call.

3. It verifies that the private call requests include:

   - All requests from the previous iteration's public inputs excluding the top one.
   - All requests present in the private call data, appended to the above in **reverse** order.

   > Ensuring the chronological execution of call requests is vital, requiring them to be arranged in reverse order. This becomes particularly crucial when calling a contract deployed earlier within the same transaction.

#### Verifying the constant data.

It verifies that the constant data matches the one in the previous iteration's public inputs.

## Private Inputs

### Previous Kernel

The data of the previous kernel iteration:

- Proof of the kernel circuit. It must be one of the following:
  - [Initial private kernel circuit](./private-kernel-initial.md).
  - Inner private kernel circuit.
  - [Reset private kernel circuit](./private-kernel-reset.md).
- Public inputs of the proof.
- Verification key of the kernel circuit.
- Membership witness for the verification key.

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

- Index of the new contract.
- Membership witness for the function leaf.
- Membership witness for the contract leaf.
- Transient note nullifier counters.

## Public Inputs

The structure of this public inputs aligns with that of the [initial private kernel circuit](./private-kernel-initial.md) and the [reset private kernel circuit](./private-kernel-reset.md).

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

It contains data accumulated during the execution of the transaction up to this point:

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
