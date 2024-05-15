# Public Kernel Circuit - Initial

:::danger
The public kernel circuits are being redesigned to accommodate the latest AVM designs. This page is therefore highly likely to change significantly.
:::

## Requirements

The **initial** public kernel iteration undergoes processes to prepare the necessary data for the executions of the public function calls.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel).

The preceding proof can only be:

- [Tail private kernel proof](./private-kernel-tail.md).

### Public Inputs Data Reset

#### Recalibrating counters.

While the counters outputted from the tail private kernel circuit preserve the correct ordering of the _public_call_requests_, they do not reflect the actual number of side effects each public call entails. This circuit allows the recalibration of counters for _public_call_requests_, ensuring subsequent public kernels can be executed with the correct counter range.

For each _request_ at index _i_ in the _public_call_requests_ within [`public_inputs`](#public-inputs).[`.transient_accumulated_data`](./public-kernel-tail#transientaccumulateddata):

1. Its hash must match the corresponding item in the _public_call_requests_ within the previous kernel's public inputs:
   - `request.hash == private_inputs.previous_kernel_public_inputs.public_call_requests[i].hash`
2. Its `counter_end` must be greater than its `counter_start`.
3. Its `counter_start` must be greater than the `counter_end` of the item at index `i + 1`.
4. If it's the last item, its `counter_start` must be `1`.

> It's crucial for the `counter_start` of the last item to be `1`, as it's assumed in the [tail public kernel circuit](./public-kernel-tail#grouping-storage-writes) that no storage writes have a counter `1`.

### Validating Public Inputs

#### Verifying the accumulated data.

It ensures that the `accumulated_data` in the [`public_inputs`](#public-inputs) matches the `accumulated_data` in [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./public-kernel-tail#public-inputs).

#### Verifying the transient accumulated data.

It ensures that all data in the [`transient_accumulated_data`](./public-kernel-tail#transientaccumulateddata) within [`public_inputs`](#public-inputs) is empty, with the exception of the `public_call_requests`.

The values in `public_call_requests` are verified in a [previous step](#recalibrating-counters).

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.mdx#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## `PrivateInputs`

### `PreviousKernel`

The format aligns with the [PreviousKernel](./private-kernel-tail.md#previouskernel)` of the tail public kernel circuit.

## `PublicInputs`

The format aligns with the [`PublicInputs`](./public-kernel-tail#public-inputs)` of the tail public kernel circuit.
