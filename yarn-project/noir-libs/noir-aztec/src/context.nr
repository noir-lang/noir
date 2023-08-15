use crate::constants_gen::{
    EMPTY_NULLIFIED_COMMITMENT,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    MAX_READ_REQUESTS_PER_CALL,
    NUM_FIELDS_PER_SHA256,
    RETURN_VALUES_LENGTH,
};

use crate::abi;

use crate::abi::{
    hash_args,
    CallContext,
    ContractDeploymentData,
    HistoricBlockData,
    FunctionData,
    PrivateCircuitPublicInputs,
    PublicCircuitPublicInputs,
};

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
// use dep::std::collections::vec::Vec;

// l1 to l2 messaging
use crate::messaging::process_l1_to_l2_message;
use crate::private_call_stack_item::PrivateCallStackItem;
use crate::public_call_stack_item::PublicCallStackItem;

use crate::types::{
    vec::BoundedVec,
    point::Point,
};

use crate::utils::arr_copy_slice;

use crate::oracle::{
    arguments,
    call_private_function::call_private_function_internal,
    public_call::call_public_function_internal,
    enqueue_public_function_call::enqueue_public_function_call_internal,
    context::get_portal_address,
};


// When finished, one can call .finish() to convert back to the abi
struct PrivateContext {
    inputs: abi::PrivateContextInputs,

    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<Field, MAX_READ_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,
    nullified_commitments: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack : BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    block_data: HistoricBlockData,

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    // encrypted_logs_preimages: Vec<Field>,
    // unencrypted_logs_preimages: Vec<Field>,
}

impl PrivateContext {
    fn new(inputs: abi::PrivateContextInputs, args_hash: Field) -> PrivateContext {
        PrivateContext {
            inputs: inputs,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            read_requests: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),
            nullified_commitments: BoundedVec::new(0),

            block_data: inputs.block_data,

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),

            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
            // encrypted_logs_preimages: Vec::new(),
            // unencrypted_logs_preimages: Vec::new(),
        }
    }

    fn msg_sender(self) -> Field {
        self.inputs.call_context.msg_sender
    }

    fn this_address(self) -> Field {
        self.inputs.call_context.storage_contract_address
    }

    fn this_portal_address(self) -> Field {
        self.inputs.call_context.portal_contract_address
    }

    fn chain_id(self) -> Field {
        self.inputs.private_global_variables.chain_id
    }

    fn version(self) -> Field {
        self.inputs.private_global_variables.version
    }

    fn finish(self) -> abi::PrivateCircuitPublicInputs {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        let encrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let unencrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let encrypted_log_preimages_length = 0;
        let unencrypted_log_preimages_length = 0;

        let priv_circuit_pub_inputs = abi::PrivateCircuitPublicInputs {
            call_context: self.inputs.call_context,
            args_hash: self.args_hash,
            return_values: self.return_values.storage,
            read_requests: self.read_requests.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            nullified_commitments: self.nullified_commitments.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: encrypted_logs_hash,
            unencrypted_logs_hash: unencrypted_logs_hash,
            encrypted_log_preimages_length: encrypted_log_preimages_length,
            unencrypted_log_preimages_length: unencrypted_log_preimages_length,
            block_data: self.block_data,
            contract_deployment_data: self.inputs.contract_deployment_data,
            chain_id: self.inputs.private_global_variables.chain_id,
            version: self.inputs.private_global_variables.version,
        };
        priv_circuit_pub_inputs
    }

    fn push_read_request(&mut self, read_request: Field) {
        self.read_requests.push(read_request);
    }

    fn push_new_note_hash(&mut self, note_hash: Field) {
        self.new_commitments.push(note_hash);
    }

    fn push_new_nullifier(&mut self, nullifier: Field, nullified_commitment: Field) {
        self.new_nullifiers.push(nullifier);
        self.nullified_commitments.push(nullified_commitment);
    }

    fn message_portal(&mut self, content: Field) {
        self.new_l2_to_l1_msgs.push(content);
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    fn consume_l1_to_l2_message(&mut self, inputs: abi::PrivateContextInputs, msg_key: Field, content: Field, secret: Field) {
        let nullifier = process_l1_to_l2_message(inputs.block_data.l1_to_l2_messages_tree_root, inputs.call_context.storage_contract_address, msg_key, content, secret);

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT)
    }

    fn accumulate_encrypted_logs<N>(&mut self, log: [Field; N]) {
        let _void1 = self.inputs;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    fn accumulate_unencrypted_logs<T>(&mut self, log: T) {
        let _void1 = self.inputs;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    fn call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: Field, 
        function_selector: Field, 
        args: [Field; ARGS_COUNT]
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash)
    }

    fn call_private_function_no_args(
        &mut self,
        contract_address: Field, 
        function_selector: Field, 
    ) -> [Field; RETURN_VALUES_LENGTH] {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0)
    }

    fn call_private_function_with_packed_args(
        &mut self,
        contract_address: Field,
        function_selector: Field,
        args_hash: Field
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let fields = call_private_function_internal(
            contract_address, 
            function_selector, 
            args_hash
        );
        let item = PrivateCallStackItem {
            contract_address: fields[0],
            function_data: FunctionData {
                function_selector: fields[1],
                is_internal: fields[2] as bool,
                is_private: fields[3] as bool,
                is_constructor: fields[4] as bool,
            },
            public_inputs: PrivateCircuitPublicInputs {
                call_context: CallContext {
                    msg_sender : fields[5],
                    storage_contract_address : fields[6],
                    portal_contract_address : fields[7],
                    
                    is_delegate_call : fields[8] as bool,
                    is_static_call : fields[9] as bool,
                    is_contract_deployment: fields[10] as bool,
                },
                // TODO handle the offsets as a variable incremented during extraction?
                args_hash: fields[11],
                return_values: arr_copy_slice(fields, [0; RETURN_VALUES_LENGTH], 12),
                read_requests: arr_copy_slice(fields, [0; MAX_READ_REQUESTS_PER_CALL], 16),
                new_commitments: arr_copy_slice(fields, [0; MAX_NEW_COMMITMENTS_PER_CALL], 20),
                new_nullifiers: arr_copy_slice(fields, [0; MAX_NEW_NULLIFIERS_PER_CALL], 24),
                nullified_commitments: arr_copy_slice(fields, [0; MAX_NEW_NULLIFIERS_PER_CALL], 28),
                private_call_stack: arr_copy_slice(fields, [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL], 32),
                public_call_stack: arr_copy_slice(fields, [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL], 36),
                new_l2_to_l1_msgs: arr_copy_slice(fields, [0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL], 40),
                encrypted_logs_hash: arr_copy_slice(fields, [0; NUM_FIELDS_PER_SHA256], 42),
                unencrypted_logs_hash: arr_copy_slice(fields, [0; NUM_FIELDS_PER_SHA256], 44),
                encrypted_log_preimages_length: fields[46],
                unencrypted_log_preimages_length: fields[47],
                block_data: HistoricBlockData {
                    // Must match order in `private_circuit_public_inputs.hpp`
                    private_data_tree_root : fields[48],
                    nullifier_tree_root : fields[49],
                    contract_tree_root : fields[50],
                    l1_to_l2_messages_tree_root : fields[51],
                    blocks_tree_root : fields[52],
                    public_data_tree_root: fields[53],
                    global_variables_hash: fields[54],
                },
                contract_deployment_data: ContractDeploymentData {
                    deployer_public_key: Point::new(fields[55], fields[56]),
                    constructor_vk_hash : fields[57],
                    function_tree_root : fields[58],
                    contract_address_salt : fields[59],
                    portal_contract_address : fields[60],
                },
                chain_id: fields[61],
                version: fields[62],
            },
            is_execution_request: fields[63] as bool,
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
        assert(item.public_inputs.call_context.msg_sender == self.inputs.call_context.storage_contract_address);
        assert(item.public_inputs.call_context.storage_contract_address == contract_address);

        self.private_call_stack.push(item.hash());

        item.public_inputs.return_values
    }

    fn call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: Field, 
        function_selector: Field, 
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash)
    }

    fn call_public_function_no_args(
        &mut self,
        contract_address: Field, 
        function_selector: Field,
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0)
    }

    fn call_public_function_with_packed_args(
        &mut self,
        contract_address: Field,
        function_selector: Field,
        args_hash: Field
    ) {
        let fields = enqueue_public_function_call_internal(
            contract_address, 
            function_selector, 
            args_hash
        );
        let item = PublicCallStackItem {
            contract_address: fields[0],
            function_data: FunctionData {
                function_selector: fields[1],
                is_internal: fields[2] as bool,
                is_private: fields[3] as bool,
                is_constructor: fields[4] as bool,
            },
            public_inputs: PublicCircuitPublicInputs {
                call_context: CallContext {
                    msg_sender : fields[5],
                    storage_contract_address : fields[6],
                    portal_contract_address : fields[7],
                    
                    is_delegate_call : fields[8] as bool,
                    is_static_call : fields[9] as bool,
                    is_contract_deployment: fields[10] as bool,
                },
                args_hash: fields[11],
                return_values: [0; RETURN_VALUES_LENGTH],
                contract_storage_update_requests: [ContractStorageUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
                contract_storage_read: [ContractStorageRead::empty(); MAX_PUBLIC_DATA_READS_PER_CALL],
                public_call_stack: [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
                new_commitments: [0; MAX_NEW_COMMITMENTS_PER_CALL],
                new_nullifiers: [0; MAX_NEW_NULLIFIERS_PER_CALL],
                new_l2_to_l1_msgs:[0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
                unencrypted_logs_hash:[0; NUM_FIELDS_PER_SHA256],
                unencrypted_log_preimages_length: 0,
                block_data: HistoricBlockData::empty(),
                prover_address: 0,
            },
            is_execution_request: true,
        };

        assert(contract_address == item.contract_address);
        assert(function_selector == item.function_data.function_selector);
        
        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        // We are issuing a regular call which is not delegate, static, or deployment. We also constrain
        // the msg_sender in the nested call to be equal to our address, and the execution context address
        // for the nested call to be equal to the address we actually called.
        assert(item.public_inputs.call_context.is_delegate_call == false);
        assert(item.public_inputs.call_context.is_static_call == false);
        assert(item.public_inputs.call_context.is_contract_deployment == false);
        assert(item.public_inputs.call_context.msg_sender == self.inputs.call_context.storage_contract_address);
        assert(item.public_inputs.call_context.storage_contract_address == contract_address);

        self.public_call_stack.push(item.hash());
    }
}

use crate::abi::{
    ContractStorageRead,
    ContractStorageUpdateRequest
};

struct PublicContext {
    inputs: abi::PublicContextInputs,

    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    contract_storage_update_requests: BoundedVec<ContractStorageUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>,
    contract_storage_read: BoundedVec<ContractStorageRead, MAX_PUBLIC_DATA_READS_PER_CALL>,
    public_call_stack: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<Field, crate::abi::MAX_NEW_NULLIFIERS_PER_CALL>,

    new_l2_to_l1_msgs: BoundedVec<Field, crate::abi::MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    unencrypted_logs_hash: BoundedVec<Field, NUM_FIELDS_PER_SHA256>,
    unencrypted_logs_preimages_length: Field,

    block_data: HistoricBlockData,
    prover_address: Field,
}

impl PublicContext {
    fn new(inputs: abi::PublicContextInputs, args_hash: Field) -> PublicContext {
        let empty_storage_read = ContractStorageRead::empty();
        let empty_storage_update = ContractStorageUpdateRequest::empty();
        PublicContext {
            inputs: inputs,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            contract_storage_update_requests: BoundedVec::new(empty_storage_update),
            contract_storage_read: BoundedVec::new(empty_storage_read),
            public_call_stack: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            new_l2_to_l1_msgs: BoundedVec::new(0),

            
            unencrypted_logs_hash: BoundedVec::new(0),
            unencrypted_logs_preimages_length: 0,

            block_data: inputs.block_data,
            prover_address: 0,

            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
            // encrypted_logs_preimages: Vec::new(),
            // unencrypted_logs_preimages: Vec::new(),
        }
    }

    fn msg_sender(self) -> Field {
        self.inputs.call_context.msg_sender
    }

    fn this_address(self) -> Field {
        self.inputs.call_context.storage_contract_address
    }

    fn this_portal_address(self) -> Field {
        self.inputs.call_context.portal_contract_address
    }

    fn chain_id(self) -> Field {
        self.inputs.public_global_variables.chain_id
    }

    fn version(self) -> Field {
        self.inputs.public_global_variables.version
    }

    fn block_number(self) -> Field {
        self.inputs.public_global_variables.block_number
    }

    fn timestamp(self) -> Field {
        self.inputs.public_global_variables.timestamp
    }

    fn finish(self) -> abi::PublicCircuitPublicInputs {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        let unencrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let unencrypted_log_preimages_length = 0;


        // Compute the public call stack hashes
        let pub_circuit_pub_inputs = abi::PublicCircuitPublicInputs {
            call_context: self.inputs.call_context, // Done
            args_hash: self.args_hash, // Done
            contract_storage_update_requests: self.contract_storage_update_requests.storage,
            contract_storage_read: self.contract_storage_read.storage,
            return_values: self.return_values.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            unencrypted_logs_hash: unencrypted_logs_hash,
            unencrypted_log_preimages_length: unencrypted_log_preimages_length,
            block_data: self.inputs.block_data,
            prover_address: self.prover_address,
        };
        pub_circuit_pub_inputs
    }

    fn push_new_note_hash(&mut self, note_hash: Field) {
        self.new_commitments.push(note_hash);
    }

    fn push_new_nullifier(&mut self, nullifier: Field, _nullified_commitment: Field) {
        self.new_nullifiers.push(nullifier);
    }

    fn message_portal(&mut self, msg: Field) {
        self.new_l2_to_l1_msgs.push(msg);
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    fn consume_l1_to_l2_message(&mut self, msg_key: Field, content: Field, secret: Field) {
        let this = (*self).this_address();
        let nullifier = process_l1_to_l2_message(self.block_data.l1_to_l2_messages_tree_root, this, msg_key, content, secret);

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT)
    }

    fn accumulate_encrypted_logs<N>(&mut self, log: [Field; N]) {
        let _void1 = self;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    fn accumulate_unencrypted_logs<T>(&mut self, log: T) {
        let _void1 = self;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    fn call_public_function<ARGS_COUNT>(
        _self: Self,
        contract_address: Field, 
        function_selector: Field,
        args: [Field; ARGS_COUNT],
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let args_hash = abi::hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        call_public_function_internal(
            contract_address, 
            function_selector, 
            args_hash,
        )
    }

    fn call_public_function_no_args(
        _self: Self,
        contract_address: Field, 
        function_selector: Field,
    ) -> [Field; RETURN_VALUES_LENGTH] {
        call_public_function_internal(
            contract_address, 
            function_selector, 
            0,
        )
    }

}