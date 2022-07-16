In these examples, we omit any 'other logic' and focus on reading a public state. Therefore the code shown might not make much sense as a useful contract, since it won't do anything except read a single state.

# High-level

```js
// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {

    uint x; // a PUBLIC state at storage slot 1 of this contract.
    uint y; // a PUBLIC state at storage slot 2 of this contract.

    function my_public_function() {
        x += y; // so we _read_ y (and _edit_ x).
    }
}
```
See the other extreme [below](#mycontract1).


# Low-level

## `my_contract_1`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn my_private_function(
    PRIVATE_INPUTS: {},
    PUBLIC_INPUTS: {
        customPublicInputs: [],
        emittedPublicInputs: {},
        executedCallback: {},

        state_transitions: [
            [x_storage_slot, x_old_value, x_new_value],
            // The old and new values will be provided by the rollup provider at the
            // time of execution.
        ],
        state_reads: [
            [y_storage_slot, y_current_value],
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
    let amount = PUBLIC_INPUTS.custom_inputs.amount;

// Perform the calculation:
    require(x_storage_slot == 1);
    require(y_storage_slot == 2);
    let x_new_value_check = x_old_value + y_current_value; // this is the read.
    require(x_new_value_check == x_new_value);

// That's it. The public _kernel_ circuit is the circuit which does membership
// checks of the current value in the public data tree.
}
```