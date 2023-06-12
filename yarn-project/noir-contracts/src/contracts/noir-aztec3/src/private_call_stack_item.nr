use crate::abi::CallContext;
use crate::abi::ContractDeploymentData;
use crate::context::PrivateFunctionContext;
use crate::abi::PrivateCircuitPublicInputs;
use crate::abi::CommitmentTreesRoots;
use crate::types::vec::BoundedVec;
use crate::utils::arr_copy_slice;

use crate::oracle::call_private_function::call_private_function_internal;


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
    is_execution_request: bool,
}

impl PrivateCallStackItem {
    fn call(contract_address: Field, function_selector: Field, args: [Field; crate::abi::MAX_ARGS], mut context: PrivateFunctionContext) -> (Self, PrivateFunctionContext) {
        let fields = call_private_function_internal(
            contract_address, 
            function_selector, 
            args
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
                    is_contract_deployment: fields[4] as bool,
                    is_delegate_call : fields[5] as bool,
                    is_static_call : fields[6] as bool,

                    msg_sender : fields[7],
                    storage_contract_address : fields[8],
                    portal_contract_address : fields[9],
                },
                // TODO handle the offsets as a variable incremented during extraction?
                args_hash: fields[10],
                return_values: arr_copy_slice(fields, [0; crate::abi::MAX_RETURN_VALUES], 11),
                new_commitments: arr_copy_slice(fields, [0; crate::abi::MAX_NEW_COMMITMENTS], 15),
                new_nullifiers: arr_copy_slice(fields, [0; crate::abi::MAX_NEW_NULLIFIERS], 19),
                private_call_stack: arr_copy_slice(fields, [0; crate::abi::MAX_PRIVATE_CALL_STACK], 23),
                public_call_stack: arr_copy_slice(fields, [0; crate::abi::MAX_PUBLIC_CALL_STACK], 27),
                new_l2_to_l1_msgs:arr_copy_slice(fields, [0; crate::abi::MAX_L2_TO_L1_MSGS], 31),
                // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
                // encrypted_logs_hash:arr_copy_slice(fields, [0; 2], 44),
                // unencrypted_logs_hash:arr_copy_slice(fields, [0; 2], 46),
                // encrypted_log_preimages_length: fields[47],
                // unencrypted_log_preimages_length: fields[48],
                commitment_trees_roots: CommitmentTreesRoots {
                    // Must match order in `private_circuit_public_inputs.hpp`
                    private_data_tree_root : fields[33],
                    nullifier_tree_root : fields[34],
                    contract_tree_root : fields[35],
                    l1_to_l2_messages_tree_root : fields[36],
                },
                contract_deployment_data: ContractDeploymentData {
                    constructor_vk_hash : fields[37],
                    function_tree_root : fields[38],
                    contract_address_salt : fields[39],
                    portal_contract_address : fields[40],
                },
            },
            is_execution_request: fields[41] as bool,
        };
        constrain contract_address == item.contract_address;
        constrain function_selector == item.function_data.function_selector;

        // TODO hash_index
        // let args_hash = dep::std::hash::pedersen(args)[0];
        // constrain args_hash == item.public_inputs.args_hash;

        constrain item.is_execution_request == false;

        constrain item.public_inputs.call_context.is_delegate_call == false;
        constrain item.public_inputs.call_context.is_static_call == false;
        constrain item.public_inputs.call_context.is_contract_deployment == false;

        context.private_call_stack = context.private_call_stack.push(item.hash());

        (item, context)
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen([
            // TODO hash_index
            // FIXME - https://github.com/AztecProtocol/aztec-packages/issues/499
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ])[0]
    }
}