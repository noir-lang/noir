# Public Kernel Circuit - Initial

## Requirements

The **initial** public kernel iteration undergoes processes to prepare the necessary data for the executions of the public function calls.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can only be:

- [Tail private kernel proof](./private-kernel-tail.md).

### Public Inputs Data Reset

#### Recalibrating counters.

While the counters outputted from the tail private kernel circuit preserve the correct ordering of the _public_call_requests_, they do not reflect the actual number of side effects each public call entails. This circuit allows the recalibration of counters for _public_call_requests_, ensuring subsequent public kernels can be executed with the correct counter range.

For each _request_ at index _i_ in the _public_call_requests_ within _[public_inputs](#public-inputs).[transient_accumulated_data](./public-kernel-tail.md#transientaccumulateddata)_:

1. Its hash must match the corresponding item in the _public_call_requests_ within the previous kernel's public inputs:
   - _`request.hash == private_inputs.previous_kernel_public_inputs.public_call_requests[i].hash`_
2. Its _counter_end_ must be greater than its _counter_start_.
3. Its _counter_start_ must be greater than the _counter_end_ of the item at index _i + 1_.
4. If it's the last item, its _counter_start_ must be _1_.

> It's crucial for the _counter_start_ of the last item to be _1_, as it's assumed in the [tail public kernel circuit](./public-kernel-tail.md#grouping-storage-writes) that no storage writes have a counter _1_.

### Validating Public Inputs

#### Verifying the accumulated data.

It ensures that the _accumulated_data_ in the _[public_inputs](#public-inputs)_ matches the _accumulated_data_ in _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./public-kernel-tail.md#public-inputs)_.

#### Verifying the transient accumulated data.

It ensures that all data in the _[transient_accumulated_data](./public-kernel-tail.md#transientaccumulateddata)_ within _[public_inputs](#public-inputs)_ is empty, with the exception of the _public_call_requests_.

The values in _public_call_requests_ are verified in a [previous step](#recalibrating-counters).

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.md#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## Private Inputs

### _PreviousKernel_

The format aligns with the _[PreviousKernel](./private-kernel-tail.md#previouskernel)_ of the tail public kernel circuit.

## Public Inputs

The format aligns with the _[Public Inputs](./public-kernel-tail.md#public-inputs)_ of the tail public kernel circuit.
