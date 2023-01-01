In these examples, we omit any 'other logic' and focus on emitting an event from a circuit.

# High-level

```js
/********************************************************************************
 * ON-CHAIN SOLIDITY:
 *******************************************************************************/

// THIS FUNCTION IS SOLIDITY, IMPORTED SOMEHOW...
// Suppose this is deployed at L1 Portal Contract address 0xabc123.
contract L1_Contract {
    function l1_to_l2_call() {
        // Code not shown here because it's a faff, but imagine this function
        // passes a callStackItemHash to the Rollup Processor.
        // See the ERC20 'deposit' example in this book for _much_ more 
        // smart contract detail.
        // Call the RollupProcessor contract:
        rollupProcessor.callL2AndPayFee(
            l2CallHash,
            callback,
            callbackOnCancel,
            l2Fee,
            l2FeeCurrency,
        );
    }

    function callback_after_l2_function_executed(
        uint256 l2CallHash,
        uint256 callIndex,
        bytes functionSignature,
        uint256[4] emittedPublicInputs, // THIS IS EVENT DATA FROM L2!
    ) {
        // Logic to validate the functionSignature, emittedPublicInputs were as
        // expected. E.g.:
        require(emittedPublicInputs[0] == 123);
        require(emittedPublicInputs[1] == 456);
        // The l2CallHash & callIndex can be used to lookup data from when the 
        // l1_to_l2_call was originally made.
    }
}

/********************************************************************************
 * CIRCUITS:
 *******************************************************************************/

// Suppose this is deployed at contract address 0xdef456.
contract My_Contract_1 {

    event MyEvent(uint param1, uint param2);

    function called_by_l1(uint a, uint b) {
        // do some stuff
        emit MyEvent(123, 456);
    }
}
```
See the other extreme [below](#mycontract1).

# Low-level

## `my_contract_1`

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn called_by_l1(
    PRIVATE_INPUTS: {
        // none shown
    },
    PUBLIC_INPUTS: {
        args: [
            a,
            b,
        ],
        emitted_events: [
            123,
            456,
        ],
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
    // Noddy example:
    let { emitted_events } = PUBLIC_INPUTS;
    assert(emitted_events[0] == 123);
    assert(emitted_publit_inputs[1] == 456);
}
```