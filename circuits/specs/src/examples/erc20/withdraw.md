# Withdraw

Withdrawal is an example of a [L2 --> L1 call](../../architecture/contracts/l1-calls.md#l2----l1-calls). A user wants to nullify a private note on L2, and then be transferred ERC20 tokens on L1.

- The user generates a 'withdraw' proof. `withdraw` is a private circuit, because it must nullify a private state.
- The `withdraw` function nullifies an `amount` owned by the caller.
- Since it's an L2-->L1 call, the `withdraw` function _does_ reveal select information.
  - It adds an `l1CallStackItem` to its `l1CallStack`.
    - This `l1CallStackItem` includes the function selector of the L1 function, as well as an encoding of the arguments to be passed to the function. In particular, two arguments for this example will be an `amount` and a `recipient`.
  - It can add callbacks, which may be executed depending on the outcome of the L1 function.  
    - In this example, the outcome will either be:
      - success: the money is transferred; or
      - fail: the L1 tx failed.
    - We don't need a 'success' callback in this example. If the money is successfully transferred, nothing else needs to happen on L2 after the L1 call succeeds (the money has already been nullified on L2).
    - We add a 'failure' callback to reinstate the private value note containing the `amount`, if the L1 tx fails.
      - This failure callback can either be a public function or private, depending on the developer's preferences. It can be public, since the 'amount' and 'recipient' are already public knowledge by this point. But it could also be private, which would then allow the user to hide that the callback has even been executed.
        - In particular, in this example, the `customPublicInputs` to the callback will need to contain the `amount` and maybe also the `recipient`, as a way of forcing the callback to only reinstate the correct amount to the correct user. Alternatively, the public input could have been the replacement commitment itself, which could have been created by the withdrawal circuit.
- Recall that the initial call to L1 must go via the Portal Contract
  - ...So the Portal Contract will receive a call from RollupProcessor.sol
  - If successful, it will return some result data (of variable size), and the RollupProcessor will sha256 hash this data (the amount of data to hash might need to be bounded to prevent griefing attacks).
    - The sha256 hash will be emitted as an L1 result (along with all other L1 results).
  - If the call fails and it doesn't return data, the RollupProcessor will register an 'L1 Result Hash' of `0`.
- The RollupProcessor with sha256 all result hashes to get a single L1ResultsHash (plural name).
- In the next rollup, the rollup provider will be _forced_ to add each L1ResultsHash to the l1ResultsTree. ('Forced', because the RollupProcessor will check the L1ResultsHash when the next rollup is submitted).
- The previously-stored callback functions can then be executed - either the success or failure callback, depending on the result.
  - Each callback function exposes the `callbackCallHash` and an `l1ResultHash`. Both of these get checked against the previously-stored data to ensure the callback is responding in a valid way to the result.
