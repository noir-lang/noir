In these examples, we omit any 'other logic' and focus on editing a public state. Therefore the code shown might not make much sense as a useful contract, since it won't do anything except edit a single state.

# High-level

```js
// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {

    uint x; // a PUBLIC state at storage slot 1 of this contract.

    function my_public_function(uint amount) {
        x += amount;
    }
}
```
See the other extreme [below](#mycontract1).

# Low-level

## `my_contract_1`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn my_public_function(
    PRIVATE_INPUTS: {},
    PUBLIC_INPUTS: {
        customPublicInputs: [
            amount,
        ],
        emittedPublicInputs: {},
        executedCallback: {},

        state_transitions: [
            [x_storage_slot, x_old_value, x_new_value],
            // The old and new values will be provided by the rollup provider at the
            // time of execution.
        ],
        state_reads: [
            // none shown in example
        ]

        private_call_stack: [],
        public_call_stack: [],
        contract_deployment_call_stack: [],
        partial_l1_call_stack: [],
        callback_stack: [],

        old_private_data_tree_root,

        is_fee_payment: false,
        pay_fee_from_l1: false,
        pay_fee_from_public_l2: false,
        called_from_l1: false,
    }
) {
    let amount = PUBLIC_INPUTS.args.amount;

// Perform the calculation:
    require(x_storage_slot == 1);
    let x_new_value_check = x_old_value + amount;
    require(x_new_value_check == x_new_value);

// That's it. The public _kernel_ circuit is the circuit which does membership
// checks of the old value and inserts the new value into the public data tree.
}
```