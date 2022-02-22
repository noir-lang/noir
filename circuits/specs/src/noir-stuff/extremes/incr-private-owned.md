# High-level

```js
contract my_contract_1 {
    secret uint x; // a SECRET state

    /**
     * @param amount - is annotated as SECRET so must NOT be exposed
     */
    function increment_my_own_private_state(secret uint amount) {
        x += amount; // knowledge of this private state's
                     // secret key is needed to make this edit.
    }
}
```
See the other extreme [below](#mycontract1).


```js
contract my_contract_2 {
    secret uint x; // a SECRET state

    /**
     * @param amount - is NOT annotated as SECRET so must be exposed
     */
    function increment_my_own_private_state(uint amount) {
        x += amount; // knowledge of this private state's
                     // secret key is needed to make this edit.
    }
}
```
See the other extreme [below](#mycontract2).

# Low-level

```js
contract my_contract_3 {
    secret uint x; // a SECRET state

    // Weird quirk: there'll be no difference in these two circuits in this case.
    function increment_my_own_private_state(secret amount) {
        x += amount; // This is inferred to be a PARTITIONED state,
                     // due to the function below.
                     // That makes this incrementation nonsensical,
                     // since _anyone_ can add a 'part' to this 
                     // state, owned by themselves.
                     // We can compile it, but it's dangerous.
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
See the other extreme [below](#mycontract3).



## `my_contract_1`:

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn increment_my_own_private_state(
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
        custom_public_inputs: {}, // lots of unused stuff denoted by {} or [].
        emitted_public_inputs: {},
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

    /*******************************************************************************
    MUTATE THE STATE
    *******************************************************************************/

    let x_new_value = x_old_value + amount;

    //******************************************************************************

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

Exactly the same as `my_contract_1.increment_my_own_private_state()`, except the `amount` parameter has not been decorated as `secret`, and so it must be made public. Therefore it gets put in `PUBLIC_INPUTS.customPublicInputs` instead of being in the `PRIVATE_INPUTS`.


## `my_contract_3`:

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn increment_my_own_private_state(
    PRIVATE_INPUTS: {
        amount, // because the param was decorated as SECRET
    
        x_owner_private_key,

        x_old_dummy_commitment,
        x_old_dummy_private_key,

        x_new_salt,
    },
    PUBLIC_INPUTS: {
        customPublicInputs: {}, // lots of unused stuff denoted by {} or [].
        emittedPublicInputs: {},
        executedCallback: {},

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
    // Calculate a proof of knowledge of secret key to permit editing of this state:
    x_owner_public_key = derive_public_key(x_owner_private_key); 

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
        custom_public_inputs: {}, // lots of unused stuff denoted by {} or [].
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