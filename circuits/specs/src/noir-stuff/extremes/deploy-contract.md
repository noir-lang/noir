This is an example of deploying a contract _from within_ another contract.

The `DeplotTest` contract deploys the `Test` contract.


# High-level


```js
// Suppose this contract will be deployed to address 0xdef456 by the DeployTest 
// contract.
contract Test {
    uint a;
    secret uint b;

    private_constructor(secret uint _b) public {
        b += _b; // Assume it's a partitioned state (so it can only be incremented
                 // or decremented).
    }

    public_constructor(uint _a) public {
        a = _a;
    }
}

// Suppose this contract is already deployed at address 0xabc123.
contract DeployTest {
    function deploy() public {
        new Test{
            salt: 0xfedcba,
            private_constructor_args: [123],
            public_constructor_args: [456],
        };
    }
}
```

`contractAddress = hash(deployerAddress, salt, vkRoot, constructorHash)`


# Low-level
## `DeployTest`

### `deploy()`

Note: because the `deploy()` function makes a call to a _private_ constructor, `deploy()` must be a private function itself (since only private functions and L1 functions can call private functions).

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn deploy(
    PRIVATE_INPUTS: {
        // This unpacked data is needed because this function passes specific 
        // constructor arguments to the constructors, and a specific salt to define
        // the new contract's address. We need this circuit to check they're 
        // included in the contract_deployment_call_stack_item_hash.
        // TODO: hashing _all_ of this data is inefficient. Eventually we'll need to
        // optimise this in the architecture.
        unpacked_contract_deployment_call_stack: [
            {   
                unpacked_data: {
                    private_constructor_public_inputs, // This preimage is needed to
                                                       // validate the constructor args.
                    public_constructor_public_inputs,   // ... and this.
                },
                call_stack_item: {
                    // Here's the call stack item for the contract deployment call.
                    // Notice: no function signature for contract deployment calls.
                    public_inputs: {
                        private_constructor_public_inputs_hash, // TODO: consider having the unpacked public inputs here instead, for ease.
                        public_constructor_public_inputs_hash,
                        private_constructor_vk_hash,
                        public_constructor_vk_hash,
                        contract_address,
                        salt,
                        vk_root,
                        circuit_data_keccak_hash, // optional, passes straight through
                                                // unchecked.
                        portal_contract_address,
                    },
                    // NOT SURE IF A CALL CONTEXT ACTUALLY NEEDS TO BE PROVIDED HERE.
                    call_context: {
                        msg_sender: 0xabc123, // The address of the entity making the call;
                                            // in this case, the DeployTest contract's
                                            // address.
                        storage_contract_address: 0xdef456, // This can be calculated in
                        // advance by the caller, since new contract addresses are
                        // deterministic.
                    },
                    is_delegate_call: false,
                    is_static_call: false,
                },
            },
        ],
    },
    PUBLIC_INPUTS: {
        custom_public_inputs: [],
        emitted_public_inputs: {},
        executed_callback: {},

        output_commitments: {},
        input_nullifiers: {},

        private_call_stack: [],
        public_call_stack: [],
        contract_deployment_call_stack: [
            test_contract_call_stack_item_hash,
        ],
        partial_l1_call_stack: [],
        callback_stack: [],

        old_private_data_tree_root,

        is_fee_payment: false,
        pay_fee_from_l1: false,
        pay_fee_from_public_l2: false,
        called_from_l1: false,
    }
) {

// Make the 'contract deployment' call:
// We need to ensure the parameters being passed to the constructors are as expected.
    let {
        unpacked_data,
        call_stack_item
    } = PRIVATE_INPUTS.unpacked_contract_deployment_call_stack[0];
    let {
        private_constructor_public_inputs_hash,
        public_constructor_public_inputs_hash,
        contract_address,
        salt,
    } = call_stack_item;
    let {
        private_constructor_public_inputs,
        public_constructor_public_inputs, 
    } = unpacked_data;

    

    // Check the correct parameters are being passed to the constructors:
    assert(private_constructor_public_inputs.custom_public_inputs[0] == 123);
    assert(public_constructor_public_inputs.custom_public_inputs[0] == 456);

    // The call contexts will be checked in the kernel snark, so we don't need
    // to check it here.

// Calculate the callStackItemHashes of the constructor calls:
    // First calculate the public input hashes:
    assert(private_constructor_public_inputs_hash == 
            ped::hash(private_constructor_public_inputs);
        // Note: this is a massive hash. TODO: We'll eventually optimise the structures to
        // to be more efficient.
    );
    assert(public_constructor_public_inputs_hash == 
            ped::hash(public_constructor_public_inputs);
        // Note: this is a massive hash. TODO: We'll eventually optimise the structures to
        // to be more efficient.
    );

    // Check the contract address uses the provided `salt = 0xfedcba`:
    assert(salt == 0xfedcba);

    assert(PUBLIC_INPUTS.contract_deployment_call_stack[0] ==
        ped::hash(call_stack_item));

    // Notice: other public inputs of the contractDeploymentCallStackItem 
    // (such as vk hashes, the vk root, the portal contract address) aren't
    // checked here. Those checks will be done by the contract deployment 
    // kernel circuit, and by users validating/reconciling the code for themselves.
}
```

### private constructor circuit

```js
fn private_constructor(
    PRIVATE_INPUTS: {
    
        b_owner_public_key,

        b_old_dummy_commitment,
        b_old_dummy_private_key,

        b_new_salt,
    },
    PUBLIC_INPUTS: {
        custom_public_inputs: [
            _b, // Although this param was decorated as `secret`, it cannot be a 
                // private input to the circuit, because it's passed into the circuit
                // by another circuit (which must validate the correctness of the
                // value being passed in).
                // It will still be hidden from the world, since the private kernel
                // circuit will swallow all custom_public_inputs, unless they're also
                // emitted as events.
                // TODO: prevent `secret` params from being exposed by an app via 
                // the `emittedPublicInputs` or call stacks, so that the kernel circuit
                // does indeed swallow the secret param with no leaks.
        ],
        emitted_public_inputs: {},
        executed_callback: {},

        output_commitments: {
            b_new_commitment,
        },
        input_nullifiers: {
            b_old_dummy_nullifier, // dummy
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
    let b_old_dummy_nullifier_check = ped::hash(
        b_old_dummy_commitment,
        b_old_dummy_private_key,
        is_dummy = true,
    )
    assert(b_old_dummy_nullifier_check == b_old_dummy_nullifier);


    // Calculate a commitment to the new 'part' of the state:
    let b_new_commitment_check = ped::hash(
        2,                      // assume `b` occupies storage slot 2.
        _b,                     // value
        b_owner_public_key,     // owner
        b_new_salt              // salt
        b_old_dummy_nullifier   // input_nullifier
    );
    assert(b_new_commitment_check == b_new_commitment);
}
```

### public constructor circuit

```rust
// The PUBLIC_INPUTS adhere to the Aztec 3 ABI for a private circuit.
fn public_constructor(
    PRIVATE_INPUTS: {},
    PUBLIC_INPUTS: {
        customPublicInputs: [
            _a,
        ],
        emittedPublicInputs: {},
        executedCallback: {},

        state_transitions: [
            [a_storage_slot, a_old_value, a_new_value],
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
    let amount = PUBLIC_INPUTS.custom_public_inputs.amount;

// Perform the calculation (for example):
    require(a_storage_slot == 1);
    let a_new_value_check = _a;
    require(a_new_value_check == a_new_value);

// That's it. The public _kernel_ circuit is the circuit which does membership
// checks of the old value and inserts the new value into the public data tree.
}
```