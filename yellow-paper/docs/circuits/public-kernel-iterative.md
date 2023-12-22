# Public Kernel Circuit - Iterative

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

In the public kernel iteration, the process involves taking a previous iteration and public call data, verifying their integrity, and preparing the necessary data for subsequent circuits to operate.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs.

The preceding proof can be:

- [tail private kernel proof](./private-kernel-tail.md).
- Iterative public kernel proof.

### Processing Public Function Call

#### Ensuring the contract instance being called is deployed.

1. The nullifier representing the contract exists in the contract tree.

   - Specifically, this nullifier is the contract address siloed with the address of a precompiled deployment contract.

2. The contract is listed in the new contracts within the public inputs.

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
6. Validates that the contract address matches the address specified in the call data.

#### Ensuring the function is legitimate:

- It must be a public function.

#### Ensuring the current call matches the call request.

The top item in the previous iteration's public call requests must pertain to the current function call.

This circuit will pop the request from the stack, comparing the hash with that of the current function call.

The preimage of the hash contains:

- Contract address.
- Function data.
- Public function circuit's public inputs.

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

#### Verifying the public function proof.

It verifies that the public function was executed with the provided proof data, verification key, and the public inputs of the VM circuit. The result of the execution is specified in the public inputs, which will be used in subsequent steps to enforce the conditions they must satisfy.

#### Verifying the public inputs of the public function circuit.

It ensures the function's intention by checking the following:

- The contract address for each non-empty item in the following arrays must equal the storage contract address of the current call:
  - Note hash contexts.
  - Nullifier contexts.
  - L2-to-L1 message contexts.
  - Read requests.
  - Update requests.
- The portal contract address for each non-empty L2-to-L1 message must equal the portal contract address of the current call.

> Ensuring the alignment of the contract addresses is crucial, as it is later used to silo the value and to establish associations with values within the same contract.

If it is a static call, it must ensure that the function does not induce any state changes by verifying that the following arrays are empty:

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.
- Update requests.

#### Verifying the call requests.

For the public call requests initiated in the current function call, it ensures that for each request at index _i_:

- Its hash equals the value at index _i_ within the call request hashes array in public function circuit's public inputs.
- If the hash is not zero, its caller context must align with the call context of the current function call, including:
  - _msg_sender_
  - Storage contract address.

#### Verifying the counters.

It verifies that each relevant value is associated with a legitimate counter.

1. For the current call:

   - The _counter_end_ of the current call must be greater than its _counter_start_.
   - Both counters must match the ones defined in the top item in the previous iteration's public call requests.

2. For the public call requests:

   - The _counter_end_ of each request must be greater than its _counter_start_.
   - The _counter_start_ of the first request must be greater than the _counter_start_ of the current call.
   - The _counter_start_ of the second and subsequent requests must be greater than the _counter_end_ of the previous request.
   - The _counter_end_ of the last request must be less than the _counter_end_ of the current call.

3. For items in each ordered array created in the current call:

   - The counter of the first item much be greater than the _counter_start_ of the current call.
   - The counter of each subsequent item much be greater than the counter of the previous item.
   - The counter of the last item much be less than the _counter_end_ of the current call.

   The ordered arrays include:

   - Read requests.
   - Update requests.

### Validating Public Inputs

#### Verifying the accumulated data.

1. It ensures that the following values match those in the previous iteration's public inputs:

   - Note hashes.
   - Nullifiers.
   - L2-to-L1 messages.
   - New contracts.
   - **Encrypted** log hash.
   - **Encrypted** log length.
   - Old public data tree snapshot.
   - New public data tree snapshot.

2. It checks that the hash and the length for **unencrypted** logs are accumulated as follows:

   - New log hash = `hash(prev_hash, cur_hash)`
     - If either hash is zero, the new hash will be `prev_hash | cur_hash`
   - New log length = `prev_length + cur_length`

#### Verifying the transient accumulated data.

1. It verifies that the following values match the result of combining the values in the previous iteration's public inputs with those in the public function circuit's public inputs:

   - Note hash contexts.
   - Nullifier contexts.
   - L2-to-L1 message contexts.
   - Read requests.
   - Update requests.

2. For the newly added update requests from public function circuit's public inputs, this circuit also checks that each is associated with an override counter, provided as a hint via the private inputs. This override counter can be:

   - Zero: if the slot does not change later in the same transaction.
   - Greater than zero: if the slot is updated later in the same transaction.
     - It pertains to a subsequent update request altering the same slot. Therefor, the counter value must be greater than the counter of the update request.

   > Override counters are used in the [tail public kernel circuit](./public-kernel-tail.md) to ensure a read happens **before** the value is changed in a later update.

   > Zero serves as an indicator for an unchanged update, as this value can never act as the counter of an update request. It corresponds to the _counter_start_ of the first function call.

3. It verifies that the public call requests include:

   - All requests from the previous iteration's public inputs except for the top one.
   - All requests present in the public call data, appended to the above in **reverse** order.

#### Verifying the constant data.

It verifies that the constant data matches the one in the previous iteration's public inputs.

## Private Inputs

### Previous Kernel

The data of the previous kernel iteration:

- Proof of the kernel circuit. It could be one of the following:
  - [Tail private kernel circuit](./private-kernel-tail.md).
  - Iterative public kernel circuit.
- Public inputs of the proof.
- Verification key of the kernel circuit.
- Membership witness for the verification key.

### Public Call Data

The call data holds details about the current public function call:

- Contract address.
- Function data.
- Public call requests.
- Public function circuit public inputs.
- Proof of the public function circuit.
- Verification key.
- Hash of the function bytecode.

### Hints

Data that aids in the verifications carried out in this circuit or later iterations:

- Index of the new contract.
- Membership witness for the function leaf.
- Membership witness for the contract leaf.
- Update requests override counters.

## Public Inputs

The structure of this public inputs aligns with that of the [tail private kernel circuit](./private-kernel-tail.md) and the [tail public kernel circuit](./public-kernel-tail.md).

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

- Note hashes.
- Nullifiers.
- L2-to-L1 messages.
- New contracts.
- Log hashes.
- Log lengths.
- Old public data tree snapshot.
- New public data tree snapshot.

### Transient Accumulated Data

It includes data from the current function call, aggregated with the results from the previous iterations:

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.
- New contract contexts.
- Read requests.
- Update requests.
- Public call requests.
