# High-level

```js
contract my_contract_3 {
    secret uint x; // a SECRET state

    function increment_someone_elses_private_state(secret amount) {
        unknown x += amount; // 'unknown' means the caller doesn't
                             // necessarily need to be the _owner_
                             // of this private state, to make this 
                             // edit.
                             // The existence of 'unknown' means this
                             // state must be PARTITIONED across many
                             // UTXOs.
    }
}
```
See the other extreme [below](#mycontract1).


# Low-level

## my_contract_1

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn increment_someone_elses_private_state(
    PRIVATE_INPUTS: {
        amount, // because the param was decorated as SECRET
    
        x_owner_public_key,

        x_old_dummy_commitment,
        x_old_dummy_private_key,

        x_new_salt,
    },
    PUBLIC_INPUTS: {
        custom_inputs: {}, // lots of unused stuff denoted by {} or [].
        emitted_public_inputs: {},
        executed_callback: {},

        output_commitments: {
            x_new_commitment,
        },
        input_nullifiers: {
            x_old_dummy_nullifier,
        },

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
    // Compute dummy nullifier:
    let x_old_dummy_nullifier_check = ped::hash(
        x_old_dummy_commitment,
        x_old_dummy_private_key,
        is_dummy = true,
    )
    assert(x_old_dummy_nullifier_check == x_old_dummy_nullifier);


    // Calculate a commitment to the new 'part' of the state:
    let x_new_commitment_check = ped::hash(
        1,                   // assume `x` occupies storage slot 1.
        amount,              // value
        x_owner_public_key,  // owner
        x_new_salt           // salt
        x_old_dummy_nullifier      // input_nullifier
    );
    assert(x_new_commitment_check == x_new_commitment);
}
```