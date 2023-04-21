use crate::abi::CallContext;
use crate::abi::ContractDeploymentData;
use crate::abi::Outputs;
use crate::abi::PrivateCircuitPublicInputs;
use crate::types::vec::BoundedVec;
use crate::utils::arr_copy_slice;

// TODO unconstrained functions fail with arrays as arguments
#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: Field, 
    _function_selector: Field, 
    _arg0: Field,
    _arg1: Field,
    _arg2: Field,
    _arg3: Field,
    _arg4: Field,
    _arg5: Field,
    _arg6: Field,
    _arg7: Field,
) -> [Field; 51] {}

unconstrained fn call_private_function_internal(
    contract_address: Field, 
    function_selector: Field,
    arg0: Field,
    arg1: Field,
    arg2: Field,
    arg3: Field,
    arg4: Field,
    arg5: Field,
    arg6: Field,
    arg7: Field,
) -> [Field; 51] {
    call_private_function_oracle(
        contract_address, 
        function_selector, 
        arg0,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
        arg7,
    )
}

struct FunctionData {
    function_selector: Field,
    is_private: bool,
    is_constructor: bool,
}

impl FunctionData {
    fn hash(self) -> Field {
        dep::std::hash::pedersen([
            // TODO hash_index
            self.function_selector,
            self.is_private as Field,
            self.is_constructor as Field,
        ])[0]
    }
}

struct PrivateCallStackItem {
    contract_address: Field,
    function_data: FunctionData,
    public_inputs: PrivateCircuitPublicInputs,
}

impl PrivateCallStackItem {
    fn call(contract_address: Field, function_selector: Field, args: [Field; crate::abi::MAX_ARGS], mut outputs: Outputs) -> (Self, Outputs) {
        let fields = call_private_function_internal(
            contract_address, 
            function_selector, 
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
            args[6],
            args[7]
        );
        let item = PrivateCallStackItem {
            contract_address: fields[0],
            function_data: FunctionData {
                function_selector: fields[1],
                is_private: fields[2] as bool,
                is_constructor: fields[3] as bool,
            },
            public_inputs: PrivateCircuitPublicInputs {
                call_context: CallContext {
                    msg_sender : fields[4],
                    storage_contract_address : fields[5],
                    portal_contract_address : fields[6],

                    is_delegate_call : fields[7] as bool,
                    is_static_call : fields[8] as bool,
                    is_contract_deployment: fields[9] as bool
                },
                // TODO handle the offsets as a variable incremented during extraction?
                args: arr_copy_slice(fields, [0; crate::abi::MAX_ARGS], 10),
                return_values: arr_copy_slice(fields, [0; crate::abi::MAX_RETURN_VALUES], 18),
                emitted_events: arr_copy_slice(fields, [0; crate::abi::MAX_LOGS], 22),
                new_commitments: arr_copy_slice(fields, [0; crate::abi::MAX_NEW_COMMITMENTS], 26),
                new_nullifiers: arr_copy_slice(fields, [0; crate::abi::MAX_NEW_NULLIFIERS], 30),
                private_call_stack: arr_copy_slice(fields, [0; crate::abi::MAX_PRIVATE_CALL_STACK], 34),
                public_call_stack: arr_copy_slice(fields, [0; crate::abi::MAX_PUBLIC_CALL_STACK], 38),
                l1_msg_stack:arr_copy_slice(fields, [0; crate::abi::MAX_L1_MSG_STACK], 42),
                historic_private_data_tree_root: fields[44],
                historic_private_nullifier_tree_root: fields[45],
                historic_contract_tree_root: fields[46],
                contract_deployment_data: ContractDeploymentData {
                    constructor_vk_hash : fields[47],
                    function_tree_root : fields[48],
                    contract_address_salt : fields[49],
                    portal_contract_address : fields[50],
                },
            },
        };
        constrain contract_address == item.contract_address;
        constrain function_selector == item.function_data.function_selector;
        for i in 0..crate::abi::MAX_ARGS {
            constrain args[i] == item.public_inputs.args[i];
        }

        outputs.private_call_stack = outputs.private_call_stack.push(item.hash());

        (item, outputs)
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen([
            // TODO hash_index
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ])[0]
    }
}