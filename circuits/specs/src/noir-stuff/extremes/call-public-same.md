In these examples, we omit any 'other logic' and focus on making a public call. Therefore the code shown might not make much sense as a useful contract, since it won't do anything except call another function.

# High-level

```js
// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {
    // vkIndex = 0
    function my_public_function(uint a, uint b, uint c) {
        // do a thing
    }

    // vkIndex = 1
    function my_private_function(uint a, uint b) {
        uint c = a + b;

        this.my_public_function(a, b, c);
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
        unpacked_public_call_stack: [
            {
                // Here's the call stack item for the call to `my_public_function`
                function_signature: {
                    contract_address: 0, // `0` is understood by the private kernel
                                         // snark to mean `address(this)`. The 
                                         // correct address will be inserted here
                                         // in the private kernel snark.
                    vk_index: 0,
                    is_private: false,
                    is_contract_deployment: false,
                    is_callback: false,
                },
                public_inputs: {
                    // All of the public inputs of a public function call
                    customPublicInputs: [
                        a,
                        b,
                        c,
                    ],
                    emittedPublicInputs: {},
                    executedCallback: {},

                    state_transitions: [
                        // none shown in example
                    ],
                    state_reads: [
                        // none shown in example
                    ]

                    public_call_stack: [],
                    contract_deployment_call_stack: [],
                    partial_l1_call_stack: [],
                    callback_stack: [],

                    old_private_data_tree_root,

                    is_fee_payment: false,
                    pay_fee_from_l1: false,
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
        customPublicInputs: [
            a,
            b,
        ],
        emittedPublicInputs: {},
        executedCallback: {},

        output_commitments: {
            // none shown in example
        },
        input_nullifiers: {
            // none shown in example
        },

        private_call_stack: [],
        public_call_stack: [
            my_public_function_call_stack_item_hash,
        ],
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
    let call_stack_item = PRIVATE_INPUTS.unpacked_public_call_stack[0];

    // Check the correct contract address is being called.
    // address(this) is a special case: we put 0 in this circuit, and the kernel
    // circuit will insert the correct address.
    assert(call_stack_item.function_signature.contract_address == 0);

    // Check the vkIndex (which can be inferred from the ordering of functions in
    // this contract).
    assert(call_stack_item.function_signature.vkIndex == 0);

    // Check the correct parameters are being passed to the function:
    assert(call_stack_item.public_inputs.custom_inputs[0] == a);
    assert(call_stack_item.public_inputs.custom_inputs[1] == b);
    assert(call_stack_item.public_inputs.custom_inputs[2] == c);

    // The call context will be checked in the kernel snark, so we don't need
    // to check it here.

    // Calculate the callStackItemHash of the call to my_public_function:
    assert(
        PUBLIC_INPUTS.public_call_stack.               
            my_public_function_call_stack_item_hash
        == ped::hash(call_stack_item) // Note: this is a massive hash.
                                      // We'll eventually optimise the structures to
                                      // to be more efficient.
    );
}
```