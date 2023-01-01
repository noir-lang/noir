# High-level

```js
contract my_contract_1 {
    secret uint x; // a SECRET state

    /**
     * @param amount - is annotated as SECRET so must NOT be exposed
     */
    function decrement_my_own_private_state(secret uint amount) {
        x -= amount; // knowledge of this private state's
                     // secret key is needed to make this edit.
    }
}
```
See the other extreme [below](#mycontract1).



```js
contract my_contract_2 {
    secret uint x; // a SECRET state

    function decrement_my_own_private_state(secret amount) {
        x -= amount; // This is inferred to be a PARTITIONED state,
                     // due to the function below.
                     // Hence, we'll nullify up-to 2 part-states
                     // and create a new part-state equal to `x - amount`
    }

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
See the other extreme [below](#mycontract2).


# Low-level

## `my_contract_1`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn decrement_my_own_private_state(
    PRIVATE_INPUTS: {
        amount, // because the param was decorated as SECRET
    
        x_owner_private_key,

        x_old_value,
        x_old_salt,
        x_old_input_nullifier,
        x_old_leaf_index,
        x_old_sibling_path,

        x_new_salt,
    },
    PUBLIC_INPUTS: {
        args: {}, // lots of unused stuff denoted by {} or [].
        emitted_events: {},
        executed_callback: {},

        output_commitments: {
            x_new_commitment,
        },
        input_nullifiers: {
            x_old_nullifier,
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

    /********************************************************************************
     * NULLIFY OLD STATE
     *******************************************************************************/

    // Calculate a proof of knowledge of secret key to permit editing of this state:
    x_owner_public_key = derive_public_key(x_owner_private_key); 

    // Calculate commitment of the old value:
    let x_old_commitment = ped::hash(
        1,                     // assume `x` occupies storage slot 1.
        x_old_value,           // value
        x_owner_public_key,    // owner
        x_old_salt,            // salt
        x_old_input_nullifier  // input_nullifier of some prev tx (to ensure uniqueness)
    );

    // Check x_old_commitment exists in the old tree:
    merkle_check(
        old_private_data_tree_root,
        x_old_commitment,
        x_old_leaf_index,
        x_old_sibling_path,
    );

    // Nullify x_old_value:
    let x_old_nullifier_check = ped::hash(
        x_old_commitment,
        x_owner_private_key
    )
    assert(x_old_nullifier_check == x_old_nullifier);

    /********************************************************************************
     * MUTATE THE STATE
     *******************************************************************************/

    let x_new_value = x_old_value - amount;

    /********************************************************************************
     * PUSH NEW STATE
     *******************************************************************************/

    // Calculate commitments of the final value:
    let x_new_commitment_check = ped::hash(
        1,                   // assume `x` occupies storage slot 1.
        x_new_value,         // value
        x_owner_public_key,  // owner
        x_new_salt           // salt
        x_old_nullifier      // input_nullifier
    );
    assert(x_new_commitment_check == x_new_commitment);

}
```


## `my_contract_2`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn decrement_my_own_private_state(
    PRIVATE_INPUTS: {
        amount, // because the param was decorated as SECRET
    
        x_owner_private_key,

        x_old_value_1,
        x_old_salt_1,
        x_old_input_nullifier_1,
        x_old_leaf_index_1,
        x_old_sibling_path_1,

        x_old_value_2,
        x_old_salt_2,
        x_old_input_nullifier_2,
        x_old_leaf_index_2,
        x_old_sibling_path_2,

        x_new_salt,
    },
    PUBLIC_INPUTS: {
        args: {}, // lots of unused stuff denoted by {} or [].
        emitted_events: {},
        executed_callback: {},

        output_commitments: {
            x_new_commitment,
        },
        input_nullifiers: {
            x_old_nullifier_1,
            x_old_nullifier_2,
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

    /********************************************************************************
     * NULLIFY OLD STATE
     *******************************************************************************/

    // Calculate a proof of knowledge of secret key to permit editing of this state:
    x_owner_public_key = derive_public_key(x_owner_private_key); 

    // Calculate commitments of the old values:
    let x_old_commitment_1 = ped::hash(
        1,                       // assume `x` occupies storage slot 1.
        x_old_value_1,           // value
        x_owner_public_key,      // owner
        x_old_salt_1,            // salt
        x_old_input_nullifier_1  // input_nullifier of some prev tx (to ensure uniqueness)
    );

    let x_old_commitment_2 = ped::hash(
        1,                       // assume `x` occupies storage slot 1.
        x_old_value_2,           // value
        x_owner_public_key,      // owner
        x_old_salt_2,            // salt
        x_old_input_nullifier_2  // input_nullifier of some prev tx (to ensure uniqueness)
    );

    // Check old commitments exists in the old tree:
    merkle_check(
        old_private_data_tree_root,
        x_old_commitment_1,
        x_old_leaf_index_1,
        x_old_sibling_path_1,
    );

     merkle_check(
        old_private_data_tree_root,
        x_old_commitment_2,
        x_old_leaf_index_2,
        x_old_sibling_path_2,
    );

    // Nullify old commitments:
    let x_old_nullifier_check_1 = ped::hash(
        x_old_commitment_1,
        x_owner_private_key
    )
    assert(x_old_nullifier_check_1 == x_old_nullifier_1);

    let x_old_nullifier_check_2 = ped::hash(
        x_old_commitment_2,
        x_owner_private_key
    )
    assert(x_old_nullifier_check_2 == x_old_nullifier_2);

    /********************************************************************************
     * MUTATE THE STATE
     *******************************************************************************/

    let x_new_value = x_old_value_1 + x_old_value_2 - amount;

    /********************************************************************************
     * PUSH NEW STATE
     *******************************************************************************/

    // Calculate commitments of the final value:
    let x_new_commitment_check = ped::hash(
        1,                   // assume `x` occupies storage slot 1.
        x_new_value,         // value
        x_owner_public_key,  // owner
        x_new_salt           // salt
        x_old_nullifier_1    // input_nullifier
    );
    assert(x_new_commitment_check == x_new_commitment);

}
```