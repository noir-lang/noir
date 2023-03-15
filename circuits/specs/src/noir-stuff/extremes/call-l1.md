In these examples, we omit any 'other logic' and focus on making an L1 call. Therefore the code shown might not make much sense as a useful contract, since it won't do anything except call another function.

This is a simplified example of a defi bridge swap, via L1. The simplification is that the L2 contract only tracks a single person's balance (and is therefore useless for privacy, but still useful as a demonstration) - I didn't want to dive into mappings and `msg.sender` syntax yet.


# High-level

```js
/********************************************************************************
 * ON-CHAIN SOLIDITY:
 *******************************************************************************/

// THIS FUNCTION IS SOLIDITY, IMPORTED SOMEHOW...
// Suppose this is deployed at L1 Portal Contract address 0xabc123.
contract L1_Portal_Contract {
    function l1_swap_a_for_b(uint amount_a) {
        // do an on-chain swap of amount_a for some amount_b
        return amount_b; // This result is added to a results tree through Aztec 3's
                         // architecture.
    }
}

/********************************************************************************
 * CIRCUITS:
 *******************************************************************************/

// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {
    secret uint token_balance_a;
    secret uint token_balance_b; 

    function swap_a_for_b(uint amount_a) {
        // Compute some stuff to make it realistic (ish),
        token_balance_a -= amount_a;

        // An L2 function can _only_ make L1 calls to its own L1 Portal Contract
        // (and not to any other L1 contract).
        // Here's some pseudocode for a special 'L1Promise' type.
        // Kind of like Rust's. Kind of like handling a JS Promise.
        // Please make the syntax better! :)
        // An L1 call (from L2) always must return an L1Promise.
        L1Promise promise = L1_Portal_Contract.l1_swap_a_for_b(amount_a);

        promise.then(
            result => hide_amount_b(result[0]), // On success, the result will 
                                                // effectively be an array of values.
                                                // Note: a result value must not be 
                                                // passed to a 'secret' arg's position.
            hide_amount_a(amount_a), // On failure, nothing is returned
                                     // (not even an error message), so the failure
                                     // callback is executed with existing args.
        );
    }

    function hide_amount_b(uint amount_b) {
        token_balance_b += amount_b;
    }

    function hide_amount_a(uint amount_a) {
        token_balance_a += amount_a;
    }
}
```
See the other extreme [below](#mycontract1).

# Low level

## `my_contract_1`

### `swap_a_for_b()`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn swap_a_for_b(
    PRIVATE_INPUTS: {

        unpacked_callback_stack: [
            {
                success_callback_call: {
                    function_data,
                    public_inputs,
                    call_context,
                    is_delegate_call,
                    is_static_call,
                },
                failure_callback_call: {
                    function_data,
                    public_inputs,
                    call_context,
                    is_delegate_call,
                    is_static_call,
                },
            }
        ]
        
        token_balance_a_owner_private_key,

        token_balance_a_old_value,
        token_balance_a_old_salt,
        token_balance_a_old_input_nullifier,
        token_balance_a_old_leaf_index,
        token_balance_a_old_sibling_path,

        token_balance_a_new_salt,
    },
    PUBLIC_INPUTS: {
        args: [
            amount_a,
        ],
        emitted_events: {},
        executed_callback: {},

        output_commitments: {
            token_balance_a_new_commitment,
        },
        input_nullifiers: {
            token_balance_a_old_nullifier,
        },

        private_call_stack: [],
        public_call_stack: [],
        contract_deployment_call_stack: [],
        partial_l1_call_stack: [
            swap_a_for_b_l1_call_stack_item_hash,
        ],
        callback_stack: [
            {
                callback_public_key, // this just passes through, if in use.
                success_callback_call_hash,
                failure_callback_call_hash,
                success_result_arg_map_acc,
            }
        ],

        old_private_data_tree_root,

        is_fee_payment: false,
        pay_fee_from_l1: false,
        pay_fee_from_public_l2: false,
        called_from_l1: false,
    }
) {

    // Params:
    let amount_a = PUBLIC_INPUTS.args.amount_a
    
    /********************************************************************************
     * MAKE THE L1 CALL
     *******************************************************************************/

    // Check the correct parameters are being passed to the function.
    // Assume we have some library that can do Solidity abi encoding.
    let arg_encoding = Solidity.abi.encode(amount_a);

    // Compress the encoding (which can very in size from app to app) to be a fixed 
    // size for the kernel snark.
    // We use keccak hashing because this will need to be unpacked on-chain in order
    // to call the L1 function.
    // Notice: the function selector can be hard-coded (0xdeadbeef here).
    assert(l1_function_l1_call_stack_item_hash == keccak(0xdeadbeef, arg_encoding));

    // Validate the callbacks:
    let { 
        success_callback_call_hash,
        failure_callback_call_hash,
        success_result_arg_map_acc,
    } = PUBLIC_INPUTS.callback_stack[0];

    let { 
        success_callback_call,
        failure_callback_call,
    } = PRIVATE_INPUTS.unpacked_callback_stack[0];


    require(
        success_callback_call == {
            function_data: {
                contract_address: 0, // Special meaning: address(this)
                vk_index: 1, // hide_amount_b()
                is_private: true,
                is_contract_deployment: false,
                is_callback: true, // <-- callback!
            },
            public_inputs: {
                // Some public inputs are omitted from this object when calculating a
                // _callback's_ call hash, because they depend on the (as of yet) 
                // unknown L1 result.
                args: [
                    0, // Inputs which will be 'result' values are set as 0.
                ],
                emitted_events: {},
                // executed_callback: {...}, // Object is omitted,
                                             // since it's not known yet.
                // TODO: maybe we ONLY need the args???
                // TODO: model the claim circuit, so I understand what's needed for
                // that context.
                output_commitments: {
                    0, // depends upon the result, so set to 0.
                },
                input_nullifiers: {
                    // DANGER - race condition. This private state could be nullified
                    // by some other call from this user, before _this_ callback
                    // gets executed. We'll need the User Client to be aware of this.
                    // QUESTION: maybe this isn't needed to be specified at this time
                    // - that would prevent the race.
                    token_balance_b_old_nullifier,
                },
                // All these are omitted from a callback's call hash, because these 
                // future calls might depend upon the result (which isn't known yet):
                // private_call_stack: [],
                // public_call_stack: [],
                // contract_deployment_call_stack: [],
                // partial_l1_call_stack: [],
                // callback_stack: [],

                // old_private_data_tree_root, // Omitted, so that the very-latest
                                               // root can be used when it's executed,
                                               // for maximum privacy.

                is_fee_payment: false,
                pay_fee_from_l1: false,
                pay_fee_from_public_l2: false,
                called_from_l1: false,
            },
            call_context: {
                msg_sender, // I _think_ we can keep this here. On some occasions,
                            // for public circuits, the rollup provider might 
                            // execute the callback, but I think it can still be
                            // signed in advance by the actual user?
                storage_contract_address: 0, // for `this` contract, we put `0`.
            }
        },
        failure_callback_call == {
            function_data: {
                contract_address: 0, // Special meaning: address(this)
                vk_index: 2, // hide_amount_a()
                is_private: true,
                is_contract_deployment: false,
                is_callback: true, // <-- callback!
            },
            public_inputs: {
                args: [
                    amount_a, // All args are known for a failure callback. (There's
                              // no result being fed in).
                ],
                emitted_events: {},
                // executed_callback: {...}, // Object is omitted,
                                             // since it's not known yet.
                output_commitments: {
                    token_balance_a_new_commitment,
                },
                input_nullifiers: {
                    // DANGER - race condition. This private state could be nullified
                    // by some other call from this user, before _this_ callback
                    // gets executed. We'll need the User Client to be aware of this.
                    // QUESTION: maybe this isn't needed to be specified at this time
                    // - that would prevent the race.
                    token_balance_a_old_nullifier,
                },
                // All these are omitted from a callback's call hash. Whilst we 
                // technically _could_ include them for the failure callback,
                // for consistency with the success callback, we omit them. 
                // private_call_stack: [],
                // public_call_stack: [],
                // contract_deployment_call_stack: [],
                // partial_l1_call_stack: [],
                // callback_stack: [],

                // old_private_data_tree_root, // Omitted, so that the very-latest
                                               // root can be used when it's executed,
                                               // for maximum privacy.

                is_fee_payment: false,
                pay_fee_from_l1: false,
                pay_fee_from_public_l2: false,
                called_from_l1: false,
            },
            call_context: {
                msg_sender, // I _think_ we can keep this here. On some occasions,
                            // for public circuits, the rollup provider might 
                            // execute the callback, but I think it can still be
                            // signed in advance by the actual user?
                storage_contract_address: 0, // for `this` contract, we put `0`.
            }
        }
    )

    let failure_callback = PUBLIC_INPUTS.callback_stack.failure_callback;
    assert(
        failure_callback.function_data == {
            contract_address: 0, // special meaning: address(this)
            vk_index: 2, // hide_amount_a()
            is_private: true,
            is_contract_deployment: false,
            is_callback: true, // <-- callback!
        }
    );
    // TODO: assert public inputs, call context, etc. too.




// Everything below is as per the ./decr-private-owned.md contract_1 example.
// I.e. it's simply the decrementing of a 'whole' private state variable.

    /********************************************************************************
     * NULLIFY OLD STATE
     *******************************************************************************/

    // Calculate a proof of knowledge of secret key to permit editing of this state:
    token_balance_a_owner_public_key = derive_public_key(
        token_balance_a_owner_private_key
    ); 

    // Calculate commitment of the old value:
    let token_balance_a_old_commitment = ped::hash(
        1, // assume `token_balance_a` occupies storage slot 1.
        token_balance_a_old_value,           // value
        token_balance_a_owner_public_key,    // owner
        token_balance_a_old_salt,            // salt
        token_balance_a_old_input_nullifier  // input_nullifier of some prev tx 
                                            // (to ensure uniqueness)
    );

    // Check token_balance_a_old_commitment exists in the old tree:
    merkle_check(
        old_private_data_tree_root,
        token_balance_a_old_commitment,
        token_balance_a_old_leaf_index,
        token_balance_a_old_sibling_path,
    );

    // Nullify token_balance_a_old_value:
    let token_balance_a_old_nullifier_check = ped::hash(
        token_balance_a_old_commitment,
        token_balance_a_owner_private_key
    )
    assert(token_balance_a_old_nullifier_check == token_balance_a_old_nullifier);

    /********************************************************************************
     * MUTATE THE STATE
     *******************************************************************************/

    let token_balance_a_new_value = token_balance_a_old_value - amount_a;

    /********************************************************************************
     * PUSH NEW STATE
     *******************************************************************************/

    // Calculate commitments of the final value:
    let token_balance_a_new_commitment_check = ped::hash(
        1, // assume `token_balance_a` occupies storage slot 1.
        token_balance_a_new_value,         // value
        token_balance_a_owner_public_key,  // owner
        token_balance_a_new_salt           // salt
        token_balance_a_old_nullifier      // input_nullifier
    );
    assert(token_balance_a_new_commitment_check == token_balance_a_new_commitment);

}
```

### `hide_amount_b()`

Designed in such a way that the function doesn't have to _know_ that it's being used as a callback.

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn hide_amount_b(
    PRIVATE_INPUTS: {
    
        token_balance_b_owner_private_key,

        token_balance_b_old_value,
        token_balance_b_old_salt,
        token_balance_b_old_input_nullifier,
        token_balance_b_old_leaf_index,
        token_balance_b_old_sibling_path,

        token_balance_b_new_salt,
    },
    PUBLIC_INPUTS: {
        args: {
            amount_b,
        },
        emitted_events: {}, // lots of unused stuff denoted by {} or [].
        executed_callback: {
            // Nonzero values only if it's actually being executed as a callback.
            // These values will be passed straight through this circuit; no checks
            // needed here - the kernel circuit will deal with l1 result resolution.
            // With this approach, the function doesn't need to 'know' that it's 
            // being used as a callback. A function can therefore be executed 
            // as a regular tx, or as a callback. That's nice.
            l1_result_hash,
            l1_results_tree_leaf_index,
        },

        output_commitments: {
            token_balance_b_new_commitment,
        },
        input_nullifiers: {
            token_balance_b_old_nullifier,
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
    token_balance_b_owner_public_key = derive_public_key(
        token_balance_b_owner_private_key
    ); 

    // Calculate commitment of the old value:
    let token_balance_b_old_commitment = ped::hash(
        1,                     // assume `x` occupies storage slot 1.
        token_balance_b_old_value,           // value
        token_balance_b_owner_public_key,    // owner
        token_balance_b_old_salt,            // salt
        token_balance_b_old_input_nullifier  // input_nullifier of some prev tx
                                             // (to ensure uniqueness)
    );

    // Check token_balance_b_old_commitment exists in the old tree:
    merkle_check(
        old_private_data_tree_root,
        token_balance_b_old_commitment,
        token_balance_b_old_leaf_index,
        token_balance_b_old_sibling_path,
    );

    // Nullify token_balance_b_old_value:
    let token_balance_b_old_nullifier_check = ped::hash(
        token_balance_b_old_commitment,
        token_balance_b_owner_private_key
    )
    assert(token_balance_b_old_nullifier_check == token_balance_b_old_nullifier);

    /********************************************************************************
     * MUTATE THE STATE
     *******************************************************************************/

    let token_balance_b_new_value = token_balance_b_old_value + amount;

    /********************************************************************************
     * PUSH NEW STATE
     *******************************************************************************/

    // Calculate commitments of the final value:
    let token_balance_b_new_commitment_check = ped::hash(
        1,                   // assume `x` occupies storage slot 1.
        token_balance_b_new_value,         // value
        token_balance_b_owner_public_key,  // owner
        token_balance_b_new_salt           // salt
        token_balance_b_old_nullifier      // input_nullifier
    );
    assert(token_balance_b_new_commitment_check == token_balance_b_new_commitment);

}
```




