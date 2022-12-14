# OUT OF DATE

# Deposit


Refer to the [example portal contract](./appendix/portal-contract.md) and [example rollup contract](../erc20/appendix/rollup-processor.md) when reading this section.


A 'deposit' is an example of an [L1 function making a call to an L2 function](../../architecture/contracts/l1-calls.md#l1----l2-calls). A user might not have any money on L2 yet, so we allow for L2 fees to be paid via L1.

## High-level flow:

- User generates inputs for an L2 contract's `deposit` function.
- User calls `PortalContract.deposit`
  - Portal Contract stores some stuff.
- Portal Contract calls Rollup Processor.
  - Rollup Processor stores more stuff.
- User generates a proof of the L2 `deposit` function.
- The `deposit` function emits an event via the `emittedPublicInputs` field.
- The proof gets added to a rollup.
- The Rollup Processor receives the rollup and is able to identify and extract info from functions which were 'called by L1 functions'.
- The Rollup Processor checks the `emittedPublicInputs` against the previously-stored on-chain stuff.
- A callback function calls the Portal Contract.
- The Portal Contract changes 'pending' states to finalised (or deletes them).
- Done.

## Slightly less high-level flow

- User calls `deposit` in the Portal Contract.
- Money is transferred-to and held by the Portal Contract, in escrow.
- An L2 circuit must also be executed as an off-chain counterpart to this on-chain deposit, in order to create a private note.
- The Portal Contract makes a call to the RollupProcessor's `callL2AndPayFee` function.
- An [`l2CallHash`](../../architecture/contracts/transactions.md#callstackitemhash) representing a call to the L2 circuit is send to the RollupProcessor to store temporarily, until the L2 function is executed.
- A `callback` function will also be given to the RollupProcessor, for it to call once the L2 circuit has been executed. This callback will call a function of any other L1 contract.
  - In our example, the callback will call the Portal Contract's `depositCallback` function.
- Once the L2 circuit is executed, it will be added to a rollup, and the rollup will be submitted by the rollup provider to `RollupProcessor.processRollup`.
- The RollupProcessor will be able to identify from the rollup calldata any 'event data' emitted by functions which were 'called' by L1.
  - In this example, the RollupProcessor will be able to extract the `l2CallHash`, `functionSignature`, and `emittedPublicInputs` of the L2 'deposit' circuit.
- The `l2CallHash`, `functionSignature`, and `emittedPublicInputs` will all be validated against what was previously stored when the L1 `deposit` function was called.
  - In particular, the `emittedPublicInputs` of this example's 'deposit' circuit will contain the `amount` deposited, as a way of reconciling the L1 call with the L2 call. It might also contain the `msg.sender`.
- If all of the checks reconcile, then the rollup provider will be paid a fee (if one was provided as part of the initial L1 call).



