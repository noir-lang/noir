In these examples, we omit any 'other logic' and focus on making a private call. Therefore the code shown might not make much sense as a useful contract, since it won't do anything except call another function.

# High-level

```js
// Suppose this is deployed at contract address 0xabc123.
contract Other_Contract {
    // vkIndex = 0 in this example.
    function other_private_function(uint a, uint b, uint c) {
        // do a thing
    }
}

// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {

    function my_private_function(uint a, uint b) {
        uint c = a + b;

        Other_Contract other_contract = Other_Contract(0xabc123); // hard-coded 
                                                                  // external address
                                                                  // in this e.g.
        other_contract.other_private_function(a, b, c);
    }
}
```
See the other extreme [below](#mycontract1).


# Low-level

## `my_contract_1`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn my_private_function(
    PRIVATE_INPUTS: {
        unpacked_private_call_stack: [
            {
                // Here's the call stack item for the call to `other_private_function`
                function_signature: {
                    contract_address: 0xabc123,
                    vk_index: 0,
                    is_private: true,
                    is_contract_deployment: false,
                    is_callback: false,
                },
                public_inputs: {
                    // All of the public inputs of a private function call
                    custom_inputs: [
                        a,
                        b,
                        c,
                    ],
                    emitted_events: {},
                    executed_callback: {},

                    output_commitments: {
                        // none shown in example
                    },
                    input_nullifiers: {
                        // none shown in example
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
                call_context: {
                    msg_sender: 0xdef456,
                    storage_contract_address: 0xabc123,
                },
                is_delegate_call: false,
                is_static_call: false,
            },
        ]
    },
    PUBLIC_INPUTS: {
        custom_inputs: [
            a,
            b,
        ],
        emitted_events: {},
        executed_callback: {},

        output_commitments: {
            // none shown in example
        },
        input_nullifiers: {
            // none shown in example
        },

        private_call_stack: [
            other_contract_other_private_function_call_stack_item_hash,
        ],
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
    let a = PUBLIC_INPUTS.custom_inputs.a;
    let b = PUBLIC_INPUTS.custom_inputs.b;

    let c = a + b;

// Make the call:
    let call_stack_item = PRIVATE_INPUTS.unpacked_private_call_stack[0];

    // Check the correct contract address is being called.
    assert(call_stack_item.function_signature.contract_address == 0x123abc);

    // Check the vkIndex (which can be inferred from the ordering of functions in
    // the Other_Contract):
    assert(call_stack_item.function_signature.vk_index == 0);

    // Check the correct parameters are being passed to the function:
    assert(call_stack_item.public_inputs.custom_inputs[0] == a);
    assert(call_stack_item.public_inputs.custom_inputs[1] == b);
    assert(call_stack_item.public_inputs.custom_inputs[2] == c);

    // The call context will be checked in the kernel snark, so we don't need
    // to check it here.

    // Calculate the callStackItemHash of the call to the other_private_function:
    assert(
        PUBLIC_INPUTS.private_call_stack.               
            other_contract_other_private_function_call_stack_item_hash
        == ped::hash(call_stack_item) // Note: this is a massive hash.
                                      // We'll eventually optimise the structures to
                                      // to be more efficient.
    );
}
```