---
title: Transaction Setup and Teardown
---

# Transaction Setup and Teardown

All transactions on the Aztec network have a private component, which is processed locally, and optionally have a public component, which is processed by sequencers using the [Public VM (AVM)](../public-vm/intro.md).

Transactions are broken into distinct phases:

1. Private setup
2. Private app logic
3. Public setup
4. Public app logic
5. Public teardown
6. Base rollup

The private setup phase is used to specify what public function will be called for public teardown, and what entity will pay the transaction fee (i.e. the `fee_payer`).

The "setup" phases are "non-revertible", meaning that if execution fails, the transaction is considered invalid and cannot be included in a block.

If execution fails in the private app logic phase, the user will not be able to generate a valid proof of their private computation, so the transaction will not be included in a block.

If the execution fails in the public app logic the _side effects_ from private app logic and public app logic will be reverted, but the transaction can still be included in a block. Execution then proceeds to the public teardown phase.

If the execution fails in the public teardown phase, the _side effects_ from private app logic, public app logic, and public teardown will be reverted, but the transaction can still be included in a block. Execution then proceeds to the base rollup phase.

In the event of a failure in public app logic or teardown, the user is charged their full [gas limit](./specifying-gas-fee-info.md#gaslimits-and-teardowngaslimits) for the transaction across all dimensions.

The public teardown phase is the only phase where the final transaction fee is available to public functions. [See more](./specifying-gas-fee-info.md#gaslimits-and-teardowngaslimits).

In the base rollup, the kernel circuit injects a public data write that levies the transaction fee on the `fee_payer`.

# An example: Fee Abstraction

Consider a user, Alice, who does not have FPA but wishes to interact with the network. Suppose she has a private balance of a fictitious asset "BananaCoin" that supports public and private balances.

Suppose there is a Fee Payment Contract (FPC) that has been deployed by another user to the network. Alice can structure her transaction as follows:

0. Before the transaction, Alice creates a private authwit in her wallet, allowing the FPC to unshield a specified amount of BananaCoin from Alice's private balance to the FPC's public balance.
1. Private setup:
   - Alice calls a private function on the FPC which is exposed for public fee payment in BananaCoin.
   - The FPC checks that the amount of teardown gas Alice has allocated is sufficient to cover the gas associated with the teardown function it will use to provide a refund to Alice.
   - The FPC specifies its teardown function as the one the transaction will use.
   - The FPC enqueues a public call to itself for the public setup phase.
   - The FPC designates itself as the `fee_payer`.
2. Private app logic:
   - Alice performs an arbitrary computation in private, potentially consuming DA gas.
3. Public setup:
   - The FPC transfers the specified amount of BananaCoin from Alice to itself.
4. Public app logic:
   - Alice performs an arbitrary computation in public, potentially consuming DA and L2 gas.
5. Public teardown:
   - The FPC looks at `transaction_fee` to compute Alice's corresponding refund of BananaCoin.
   - The FPC transfers the refund to Alice via a pending shield.
6. Base rollup:
   - The Base rollup kernel circuit injects a public data write that levies the transaction fee on the `fee_payer`.

This illustrates the utility of the various phases. In particular, we see why the setup phase must not be revertible: if Alice's public app logic fails, the FPC is still going to pay the fee in the base rollup; if public setup were revertible, the transfer of Alice's BananaCoin would revert so the FPC would be losing money.

# Sequencer Whitelisting

Because a transaction is invalid if it fails in the public setup phase, sequencers are taking a risk by processing them. To mitigate this risk, it is expected that sequencers will only process transactions that use public functions that they have whitelisted.

# Defining Setup

The private function that is executed first is referred to as the "entrypoint".

Tracking which side effects belong to setup versus app logic is done by keeping track of [side effect counters](../circuits/private-kernel-initial.mdx#processing-a-private-function-call), and storing the value of the counter at which the setup phase ends within the private context.

This value is stored in the `PrivateContext` as the `min_revertible_side_effect_counter`, and is set by calling `context.end_setup()`.

This is converted into the `PrivateCircuitPublicInputs` as `min_revertible_side_effect_counter`.

Execution of the entrypoint is always verified/processed by the `PrivateKernelInit` circuit.

It is only the `PrivateKernelInit` circuit that looks at the `min_revertible_side_effect_counter` as reported by `PrivateCirclePublicInputs`, and thus it is only the entrypoint that can effectively call `context.end_setup()`.

# Defining Teardown

At any point during private execution, a contract may call `context.set_public_teardown_function` to specify a public function that will be called during the public teardown phase. This function takes the same arguments as `context.call_public_function`, but does not have a side effect counter associated with it.

Similar to `call_public_function`, this results in the hash of a `PublicCallStackItem` being set on `PrivateCircuitPublicInputs` as `public_teardown_function_hash`.

The private kernel circuits will verify that this hash is set at most once.

# Interpreting the `min_revertible_side_effect_counter`

Notes, nullifiers, and logs are examples of side effects that are partitioned into setup and app logic.

[Enqueueing a public function](../calls/enqueued-calls.md) from private is also a side effect: if the counter associated with an enqueued public function is less than the `min_revertible_side_effect_counter`, the public function will be executed during the public setup phase, otherwise it will be executed during the public app logic phase.

As mentioned above, setting the public teardown function is not a side effect.

If a transaction has enqueued public functions, or has a public teardown function, then during the PrivateKernelTailToPublic the `min_revertible_side_effect_counter` is used to partition the side effects produced during private execution into revertible and non-revertible sets on the `PublicKernelCircuitPublicInputs`, i.e. `end` and `end_non_revertible`.

The public teardown function is set on the `PublicKernelCircuitPublicInputs` as `public_teardown_function_hash`.

If a transaction does not have any enqueued public functions, and does not have a public teardown function, then the `PrivateKernelTail` is used instead of the `PrivateKernelTailToPublic`, and no partitioning is done.
