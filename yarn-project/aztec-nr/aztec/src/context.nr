use crate::{
    abi::{
        PrivateContextInputs,
        PublicContextInputs,
    },
    key::nullifier_key::validate_nullifier_key_against_address,
    messaging::process_l1_to_l2_message,
    oracle::{
        arguments,
        call_private_function::call_private_function_internal,
        public_call::call_public_function_internal,
        enqueue_public_function_call::enqueue_public_function_call_internal,
        context::get_portal_address,
        get_block_header::get_block_header,
        nullifier_key::{get_nullifier_key_pair, NullifierKeyPair},
    },
    types::vec::BoundedVec,
    utils::Reader,
};
use dep::protocol_types::{
    abis::{
        block_header::BlockHeader,
        call_context::CallContext,
        function_data::FunctionData,
        function_selector::FunctionSelector,
        nullifier_key_validation_request::NullifierKeyValidationRequest,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        public_circuit_public_inputs::PublicCircuitPublicInputs,
        call_stack_item::PrivateCallStackItem,
        call_stack_item::PublicCallStackItem,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    },
    address::{
        AztecAddress,
        EthAddress,
    },
    constants::{
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
        NUM_FIELDS_PER_SHA256,
        RETURN_VALUES_LENGTH,
    },
    contrakt::{
        deployment_data::ContractDeploymentData,
        storage_read::StorageRead,
        storage_update_request::StorageUpdateRequest,
    },
    grumpkin_point::GrumpkinPoint,
    grumpkin_private_key::GrumpkinPrivateKey,
    hash::hash_args,
};
use dep::std::{
    option::Option,
};

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
// use dep::std::collections::vec::Vec;

// When finished, one can call .finish() to convert back to the abi
struct PrivateContext {
    // docs:start:private-context
    inputs: PrivateContextInputs,
    side_effect_counter: u32,

    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<SideEffect, MAX_READ_REQUESTS_PER_CALL>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<SideEffect, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack_hashes : BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    // docs:end:private-context

    block_header: BlockHeader,

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    // encrypted_logs_preimages: Vec<Field>,
    // unencrypted_logs_preimages: Vec<Field>,

    nullifier_key: Option<NullifierKeyPair>,
}

impl PrivateContext {
    pub fn new(inputs: PrivateContextInputs, args_hash: Field) -> PrivateContext {
        PrivateContext {
            inputs: inputs,
            side_effect_counter: inputs.call_context.start_side_effect_counter,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            read_requests: BoundedVec::new(SideEffect::empty()),
            nullifier_key_validation_requests: BoundedVec::new(NullifierKeyValidationRequest::empty()),

            new_commitments: BoundedVec::new(SideEffect::empty()),
            new_nullifiers: BoundedVec::new(SideEffectLinkedToNoteHash::empty()),

            block_header: inputs.block_header,

            private_call_stack_hashes: BoundedVec::new(0),
            public_call_stack_hashes: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),

            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
            // encrypted_logs_preimages: Vec::new(),
            // unencrypted_logs_preimages: Vec::new(),

            nullifier_key: Option::none(),
        }
    }

    pub fn msg_sender(self) -> AztecAddress {
        self.inputs.call_context.msg_sender
    }

    pub fn this_address(self) -> AztecAddress {
        self.inputs.call_context.storage_contract_address
    }

    pub fn this_portal_address(self) -> EthAddress {
        self.inputs.call_context.portal_contract_address
    }

    pub fn chain_id(self) -> Field {
        self.inputs.private_global_variables.chain_id
    }

    pub fn version(self) -> Field {
        self.inputs.private_global_variables.version
    }

    pub fn selector(self) -> FunctionSelector {
        self.inputs.call_context.function_selector
    }

    pub fn get_block_header(self, block_number: u32) -> BlockHeader {
        get_block_header(block_number, self)
    }

    pub fn finish(self) -> PrivateCircuitPublicInputs {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        let encrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let unencrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let encrypted_log_preimages_length = 0;
        let unencrypted_log_preimages_length = 0;

        let priv_circuit_pub_inputs = PrivateCircuitPublicInputs {
            call_context: self.inputs.call_context,
            args_hash: self.args_hash,
            return_values: self.return_values.storage,
            read_requests: self.read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack_hashes: self.private_call_stack_hashes.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            end_side_effect_counter: self.side_effect_counter,
            encrypted_logs_hash: encrypted_logs_hash,
            unencrypted_logs_hash: unencrypted_logs_hash,
            encrypted_log_preimages_length: encrypted_log_preimages_length,
            unencrypted_log_preimages_length: unencrypted_log_preimages_length,
            block_header: self.block_header,
            contract_deployment_data: self.inputs.contract_deployment_data,
            chain_id: self.inputs.private_global_variables.chain_id,
            version: self.inputs.private_global_variables.version,
        };
        priv_circuit_pub_inputs
    }

    pub fn push_read_request(&mut self, read_request: Field) {
        let side_effect = SideEffect {
            value: read_request,
            counter: self.side_effect_counter,
        };
        self.read_requests.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn push_new_note_hash(&mut self, note_hash: Field) {
        let side_effect = SideEffect {
            value: note_hash,
            counter: self.side_effect_counter,
        };
        self.new_commitments.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn push_new_nullifier(&mut self, nullifier: Field, nullified_commitment: Field) {
        let side_effect = SideEffectLinkedToNoteHash {
            value: nullifier,
            note_hash: nullified_commitment,
            counter: self.side_effect_counter,
        };
        self.new_nullifiers.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn request_nullifier_secret_key(&mut self, account: AztecAddress) -> GrumpkinPrivateKey {
        let key_pair = if self.nullifier_key.is_none() {
            let key_pair = get_nullifier_key_pair(account);
            validate_nullifier_key_against_address(account, key_pair.public_key);
            let request = NullifierKeyValidationRequest { public_key: key_pair.public_key, secret_key: key_pair.secret_key };
            self.nullifier_key_validation_requests.push(request);
            self.nullifier_key = Option::some(key_pair);
            key_pair
        } else {
            let key_pair = self.nullifier_key.unwrap_unchecked();
            // If MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL is larger than 1, need to update the way the key pair is cached.
            assert(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL == 1);
            assert(key_pair.account == account, "Cannot query nullifier key for more than one account per call");
            key_pair
        };
        key_pair.secret_key
    }

    // docs:start:context_message_portal
    pub fn message_portal(&mut self, content: Field) 
    // docs:end:context_message_portal
    {
        self.new_l2_to_l1_msgs.push(content);
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    // docs:start:context_consume_l1_to_l2_message
    // docs:start:consume_l1_to_l2_message
    pub fn consume_l1_to_l2_message(
        &mut self,
        msg_key: Field,
        content: Field,
        secret: Field
    ) 
    // docs:end:context_consume_l1_to_l2_message
    {
        let nullifier = process_l1_to_l2_message(self.block_header.l1_to_l2_message_tree_root, self.this_address(), self.this_portal_address(), self.chain_id(), self.version(), msg_key, content, secret);

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, 0)
    }
    // docs:end:consume_l1_to_l2_message

    pub fn accumulate_encrypted_logs<N>(&mut self, log: [Field; N]) {
        let _void1 = self.inputs;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    pub fn accumulate_unencrypted_logs<T>(&mut self, log: T) {
        let _void1 = self.inputs;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    pub fn call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector, 
        args: [Field; ARGS_COUNT]
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash)
    }

    pub fn call_private_function_no_args(
        &mut self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector, 
    ) -> [Field; RETURN_VALUES_LENGTH] {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0)
    }

    pub fn call_private_function_with_packed_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let fields = call_private_function_internal(
            contract_address,
            function_selector, 
            args_hash,
            self.side_effect_counter,
        );
        let mut reader = Reader::new(fields);

        let item = PrivateCallStackItem {
            contract_address: AztecAddress::from_field(reader.read()),
            function_data: FunctionData {
                selector: FunctionSelector::from_field(reader.read()),
                is_internal: reader.read() as bool,
                is_private: reader.read() as bool,
                is_constructor: reader.read() as bool,
            },
            public_inputs: PrivateCircuitPublicInputs {
                call_context: CallContext {
                    msg_sender : AztecAddress::from_field(reader.read()),
                    storage_contract_address : AztecAddress::from_field(reader.read()),
                    portal_contract_address : EthAddress::from_field(reader.read()),
                    function_selector: FunctionSelector::from_field(reader.read()), // practically same as fields[1]
                    is_delegate_call : reader.read() as bool,
                    is_static_call : reader.read() as bool,
                    is_contract_deployment: reader.read() as bool,
                    start_side_effect_counter: reader.read() as u32,
                },
                args_hash: reader.read(),
                return_values: reader.read_array([0; RETURN_VALUES_LENGTH]), // +1
                read_requests: reader.read_struct_array(SideEffect::deserialise, [SideEffect::empty(); MAX_READ_REQUESTS_PER_CALL]),
                nullifier_key_validation_requests: reader.read_struct_array(NullifierKeyValidationRequest::deserialise, [NullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL]),
                new_commitments: reader.read_struct_array(SideEffect::deserialise, [SideEffect::empty(); MAX_NEW_COMMITMENTS_PER_CALL]),
                new_nullifiers: reader.read_struct_array(SideEffectLinkedToNoteHash::deserialise, [SideEffectLinkedToNoteHash::empty(); MAX_NEW_NULLIFIERS_PER_CALL]),
                private_call_stack_hashes: reader.read_array([0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL]),
                public_call_stack_hashes: reader.read_array([0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]),
                new_l2_to_l1_msgs: reader.read_array([0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL]),
                end_side_effect_counter: reader.read() as u32,
                encrypted_logs_hash: reader.read_array([0; NUM_FIELDS_PER_SHA256]),
                unencrypted_logs_hash: reader.read_array([0; NUM_FIELDS_PER_SHA256]),
                encrypted_log_preimages_length: reader.read(),
                unencrypted_log_preimages_length: reader.read(),
                block_header: BlockHeader{
                    // Must match order in `private_circuit_public_inputs.hpp`
                    note_hash_tree_root : reader.read(),
                    nullifier_tree_root : reader.read(),
                    contract_tree_root : reader.read(),
                    l1_to_l2_message_tree_root : reader.read(),
                    archive_root : reader.read(),
                    public_data_tree_root: reader.read(),
                    global_variables_hash: reader.read(),
                },
                contract_deployment_data: ContractDeploymentData {
                    deployer_public_key: GrumpkinPoint {
                      x: reader.read(), 
                      y: reader.read()
                    },
                    constructor_vk_hash : reader.read(),
                    function_tree_root : reader.read(),
                    contract_address_salt : reader.read(),
                    portal_contract_address : EthAddress::from_field(reader.read()),
                },
                chain_id: reader.read(),
                version: reader.read(),
            },
            is_execution_request: reader.read() as bool,
        };

        reader.finish();

        assert_eq(item.public_inputs.call_context.start_side_effect_counter, self.side_effect_counter);
        self.side_effect_counter = item.public_inputs.end_side_effect_counter + 1;
        
        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert(args_hash == item.public_inputs.args_hash);

        assert(item.is_execution_request == false);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        // We are issuing a regular call which is not delegate, static, or deployment. We also constrain
        // the msg_sender in the nested call to be equal to our address, and the execution context address
        // for the nested call to be equal to the address we actually called.
        assert(item.public_inputs.call_context.is_delegate_call == false);
        assert(item.public_inputs.call_context.is_static_call == false);
        assert(item.public_inputs.call_context.is_contract_deployment == false);
        assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address));
        assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));

        self.private_call_stack_hashes.push(item.hash());

        item.public_inputs.return_values
    }

    pub fn call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector, 
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash)
    }

    pub fn call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector,
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0)
    }

    pub fn call_public_function_with_packed_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field
    ) {
        let fields = enqueue_public_function_call_internal(
            contract_address, 
            function_selector, 
            args_hash,
            self.side_effect_counter
        );

        let mut reader = Reader::new(fields);

        let item = PublicCallStackItem {
            contract_address:  AztecAddress::from_field(reader.read()),
            function_data: FunctionData {
                selector: FunctionSelector::from_field(reader.read()),
                is_internal: reader.read() as bool,
                is_private: reader.read() as bool,
                is_constructor: reader.read() as bool,
            },
            public_inputs: PublicCircuitPublicInputs {
                call_context: CallContext {
                    msg_sender : AztecAddress::from_field(reader.read()),
                    storage_contract_address : AztecAddress::from_field(reader.read()),
                    portal_contract_address : EthAddress::from_field(reader.read()),
                    function_selector: FunctionSelector::from_field(reader.read()), // practically same as fields[1]
                    is_delegate_call : reader.read() as bool,
                    is_static_call : reader.read() as bool,
                    is_contract_deployment: reader.read() as bool,
                    start_side_effect_counter: reader.read() as u32,
                },
                args_hash: reader.read(),
                return_values: [0; RETURN_VALUES_LENGTH],
                contract_storage_update_requests: [StorageUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
                contract_storage_reads: [StorageRead::empty(); MAX_PUBLIC_DATA_READS_PER_CALL],
                public_call_stack_hashes: [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
                new_commitments: [SideEffect::empty(); MAX_NEW_COMMITMENTS_PER_CALL],
                new_nullifiers: [SideEffectLinkedToNoteHash::empty(); MAX_NEW_NULLIFIERS_PER_CALL],
                new_l2_to_l1_msgs:[0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
                unencrypted_logs_hash:[0; NUM_FIELDS_PER_SHA256],
                unencrypted_log_preimages_length: 0,
                block_header: BlockHeader::empty(),
                prover_address: AztecAddress::zero(),
            },
            is_execution_request: true,
        };
        reader.finish();

        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert_eq(item.public_inputs.call_context.start_side_effect_counter, self.side_effect_counter);
        // We increment the sideffect counter by one, to account for the call itself being a side effect.
        self.side_effect_counter = self.side_effect_counter + 1;
        
        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        // We are issuing a regular call which is not delegate, static, or deployment. We also constrain
        // the msg_sender in the nested call to be equal to our address, and the execution context address
        // for the nested call to be equal to the address we actually called.
        assert(item.public_inputs.call_context.is_delegate_call == false);
        assert(item.public_inputs.call_context.is_static_call == false);
        assert(item.public_inputs.call_context.is_contract_deployment == false);
        assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address));
        assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));

        self.public_call_stack_hashes.push(item.hash());
    }
}

struct PublicContext {
    inputs: PublicContextInputs,
    side_effect_counter: u32,

    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    contract_storage_update_requests: BoundedVec<StorageUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>,
    contract_storage_reads: BoundedVec<StorageRead, MAX_PUBLIC_DATA_READS_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,

    new_commitments: BoundedVec<SideEffect, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL>,

    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    unencrypted_logs_hash: BoundedVec<Field, NUM_FIELDS_PER_SHA256>,
    unencrypted_logs_preimages_length: Field,

    block_header: BlockHeader,
    prover_address: AztecAddress,
}

impl PublicContext {
    pub fn new(inputs: PublicContextInputs, args_hash: Field) -> PublicContext {
        let empty_storage_read = StorageRead::empty();
        let empty_storage_update = StorageUpdateRequest::empty();
        PublicContext {
            inputs: inputs,
            side_effect_counter: inputs.call_context.start_side_effect_counter,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            contract_storage_update_requests: BoundedVec::new(empty_storage_update),
            contract_storage_reads: BoundedVec::new(empty_storage_read),
            public_call_stack_hashes: BoundedVec::new(0),

            new_commitments: BoundedVec::new(SideEffect::empty()),
            new_nullifiers: BoundedVec::new(SideEffectLinkedToNoteHash::empty()),

            new_l2_to_l1_msgs: BoundedVec::new(0),

            
            unencrypted_logs_hash: BoundedVec::new(0),
            unencrypted_logs_preimages_length: 0,

            block_header: inputs.block_header,
            prover_address: AztecAddress::zero(),

            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
            // encrypted_logs_preimages: Vec::new(),
            // unencrypted_logs_preimages: Vec::new(),
        }
    }

    pub fn msg_sender(self) -> AztecAddress {
        self.inputs.call_context.msg_sender
    }

    pub fn this_address(self) -> AztecAddress {
        self.inputs.call_context.storage_contract_address
    }

    pub fn this_portal_address(self) -> EthAddress {
        self.inputs.call_context.portal_contract_address
    }

    pub fn chain_id(self) -> Field {
        self.inputs.public_global_variables.chain_id
    }

    pub fn version(self) -> Field {
        self.inputs.public_global_variables.version
    }

    pub fn selector(self) -> FunctionSelector {
        self.inputs.call_context.function_selector
    }

    pub fn block_number(self) -> Field {
        self.inputs.public_global_variables.block_number
    }

    pub fn timestamp(self) -> Field {
        self.inputs.public_global_variables.timestamp
    }

    pub fn finish(self) -> PublicCircuitPublicInputs {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        let unencrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let unencrypted_log_preimages_length = 0;


        // Compute the public call stack hashes
        let pub_circuit_pub_inputs = PublicCircuitPublicInputs {
            call_context: self.inputs.call_context, // Done
            args_hash: self.args_hash, // Done
            contract_storage_update_requests: self.contract_storage_update_requests.storage,
            contract_storage_reads: self.contract_storage_reads.storage,
            return_values: self.return_values.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            unencrypted_logs_hash: unencrypted_logs_hash,
            unencrypted_log_preimages_length: unencrypted_log_preimages_length,
            block_header: self.inputs.block_header,
            prover_address: self.prover_address,
        };
        pub_circuit_pub_inputs
    }

    pub fn push_new_note_hash(&mut self, note_hash: Field) {
        let side_effect = SideEffect {
            value: note_hash,
            counter: self.side_effect_counter
        };
        self.new_commitments.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn push_new_nullifier(&mut self, nullifier: Field, _nullified_commitment: Field) {
        let side_effect = SideEffectLinkedToNoteHash {
            value: nullifier,
            note_hash: 0, // cannot nullify pending notes in public context
            counter: self.side_effect_counter
        };
        self.new_nullifiers.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn message_portal(&mut self, content: Field) {
        self.new_l2_to_l1_msgs.push(content);
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    pub fn consume_l1_to_l2_message(&mut self, msg_key: Field, content: Field, secret: Field) {
        let this = (*self).this_address();
        let nullifier = process_l1_to_l2_message(self.block_header.l1_to_l2_message_tree_root, this, self.this_portal_address(), self.chain_id(), self.version(), msg_key, content, secret);

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, 0)
    }

    pub fn accumulate_encrypted_logs<N>(&mut self, log: [Field; N]) {
        let _void1 = self;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    pub fn accumulate_unencrypted_logs<T>(&mut self, log: T) {
        let _void1 = self;
        let _void2 = log;
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    }

    pub fn call_public_function<ARGS_COUNT>(
        _self: Self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT],
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        call_public_function_internal(
            contract_address, 
            function_selector, 
            args_hash,
        )
    }

    pub fn call_public_function_no_args(
        _self: Self,
        contract_address: AztecAddress, 
        function_selector: FunctionSelector,
    ) -> [Field; RETURN_VALUES_LENGTH] {
        call_public_function_internal(
            contract_address, 
            function_selector, 
            0,
        )
    }

}

struct Context {
    private: Option<&mut PrivateContext>,
    public: Option<&mut PublicContext>,
}

impl Context {
    pub fn private(context: &mut PrivateContext) -> Context {
        Context {
            private: Option::some(context),
            public: Option::none()
        }
    }

    pub fn public(context: &mut PublicContext) -> Context {
        Context {
            public: Option::some(context),
            private: Option::none()
        }
    }

    pub fn none() -> Context {
        Context {
            public: Option::none(),
            private: Option::none()
        }
    }
}
