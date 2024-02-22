# Static calls

[Synchronous calls](./sync-calls.md), both private and public, can be executed as _static_ calls. This means that the called function, and all nested calls within, cannot emit any modifying side effects, such as creating or consuming notes, writing to storage, or emitting events. The purpose of a static call is to query another contract while ensuring that the call will not modify state. Static calls are based on [EIP214](https://eips.ethereum.org/EIPS/eip-214).

In particular, the following fields of the returned `CallStackItem` must be zero or empty in a static call:

<!-- Please can we have a similar list for the side effects of a public call? We're missing things like public state writes. -->
<!--
What about nested calls? Is a function which is 'staticcalled' allowed to make calls?
I think the options are:
- No. Or,
- Yes, but any nested calls from a staticcalled function must also be static calls.
Thoughts? Ethereum does the latter. We should write about whichever we choose in this page.
-->

- `new_note_hashes`
- `new_nullifiers`
- `nullified_commitments`
- `new_l2_to_l1_msgs`
- `encrypted_logs_hash`
- `unencrypted_logs_hash`
- `encrypted_log_preimages_length`
- `unencrypted_log_preimages_length`

At the protocol level, a static call is identified by a `is_static_call` flag in the `CircuitPublicInputs` of the `CallStackItem`. The kernel is responsible for asserting that the call and all nested calls do not emit any forbidden side effects.

At the contract level, a caller can initiate a static call via a `staticCallPrivateFunction` or `staticCallPublicFunction` oracle call. The caller is responsible for asserting that the returned `CallStackItem` has the `is_static_call` flag correctly set.
