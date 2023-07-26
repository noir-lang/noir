use crate::abi::CallContext;
use crate::abi::hash_args;
use crate::abi::ContractDeploymentData;
use crate::context::Context;
use crate::abi::PrivateCircuitPublicInputs;
use crate::abi::CommitmentTreesRoots;
use crate::types::vec::BoundedVec;
use crate::types::point::Point;
use crate::utils::arr_copy_slice;
use crate::constants_gen;
use crate::constants_gen::GENERATOR_INDEX__FUNCTION_DATA;
use crate::constants_gen::GENERATOR_INDEX__CALL_STACK_ITEM;

use crate::oracle::call_private_function::call_private_function_internal;
use crate::oracle::arguments;

struct FunctionData {
    function_selector: Field,
    is_private: bool,
    is_constructor: bool,
}

impl FunctionData {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator([
            self.function_selector,
            self.is_private as Field,
            self.is_constructor as Field,
        ], GENERATOR_INDEX__FUNCTION_DATA)[0]
    }
}

struct PrivateCallStackItem {
    contract_address: Field,
    function_data: FunctionData,
    public_inputs: PrivateCircuitPublicInputs,
    is_execution_request: bool,
}

impl PrivateCallStackItem {
    fn call<ARGS_COUNT>(
        contract_address: Field, 
        function_selector: Field, 
        args: [Field; ARGS_COUNT], 
        mut context: Context) -> (Self, Context) {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        PrivateCallStackItem::call_with_packed_args(contract_address, function_selector, args_hash, context)
    }

    fn call_with_packed_args(contract_address: Field, function_selector: Field, args_hash: Field, mut context: Context) -> (Self, Context) {
        let fields = call_private_function_internal(
            contract_address, 
            function_selector, 
            args_hash
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
                    is_contract_deployment: fields[9] as bool,
                },
                // TODO handle the offsets as a variable incremented during extraction?
                args_hash: fields[10],
                return_values: arr_copy_slice(fields, [0; constants_gen::RETURN_VALUES_LENGTH], 11),
                read_requests: arr_copy_slice(fields, [0; constants_gen::MAX_READ_REQUESTS_PER_CALL], 15),
                new_commitments: arr_copy_slice(fields, [0; constants_gen::MAX_NEW_COMMITMENTS_PER_CALL], 19),
                new_nullifiers: arr_copy_slice(fields, [0; constants_gen::MAX_NEW_NULLIFIERS_PER_CALL], 23),
                nullified_commitments: arr_copy_slice(fields, [0; constants_gen::MAX_NEW_NULLIFIERS_PER_CALL], 27),
                private_call_stack: arr_copy_slice(fields, [0; constants_gen::MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL], 31),
                public_call_stack: arr_copy_slice(fields, [0; constants_gen::MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL], 35),
                new_l2_to_l1_msgs: arr_copy_slice(fields, [0; constants_gen::MAX_NEW_L2_TO_L1_MSGS_PER_CALL], 39),
                encrypted_logs_hash: arr_copy_slice(fields, [0; constants_gen::NUM_FIELDS_PER_SHA256], 41),
                unencrypted_logs_hash: arr_copy_slice(fields, [0; constants_gen::NUM_FIELDS_PER_SHA256], 43),
                encrypted_log_preimages_length: fields[45],
                unencrypted_log_preimages_length: fields[46],
                commitment_trees_roots: CommitmentTreesRoots {
                    // Must match order in `private_circuit_public_inputs.hpp`
                    private_data_tree_root : fields[47],
                    nullifier_tree_root : fields[48],
                    contract_tree_root : fields[49],
                    l1_to_l2_messages_tree_root : fields[50],
                    blocks_tree_root : fields[51],
                },
                contract_deployment_data: ContractDeploymentData {
                    deployer_public_key: Point::new(fields[52], fields[53]),
                    constructor_vk_hash : fields[54],
                    function_tree_root : fields[55],
                    contract_address_salt : fields[56],
                    portal_contract_address : fields[57],
                },
                chain_id: fields[58],
                version: fields[59],
            },
            is_execution_request: fields[60] as bool,
        };
        assert(contract_address == item.contract_address);
        assert(function_selector == item.function_data.function_selector);

        assert(args_hash == item.public_inputs.args_hash);

        assert(item.is_execution_request == false);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        // We are issuing a regular call which is not delegate, static, or deployment. We also constrain
        // the msg_sender in the nested call to be equal to our address, and the execution context address
        // for the nested call to be equal to the address we actually called.
        assert(item.public_inputs.call_context.is_delegate_call == false);
        assert(item.public_inputs.call_context.is_static_call == false);
        assert(item.public_inputs.call_context.is_contract_deployment == false);
        assert(item.public_inputs.call_context.msg_sender == context.inputs.call_context.storage_contract_address);
        assert(item.public_inputs.call_context.storage_contract_address == contract_address);

        context.private_call_stack = context.private_call_stack.push(item.hash());

        (item, context)
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator([
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)[0]
    }
}