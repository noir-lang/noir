use crate::{
    abi::PublicContextInputs,
    messaging::process_l1_to_l2_message,
    oracle::{
        arguments,
        public_call::call_public_function_internal,
    },
};
use dep::protocol_types::{
    abis::{
        global_variables::GlobalVariables,
        function_selector::FunctionSelector,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        public_call_stack_item::PublicCallStackItem,
        public_circuit_public_inputs::PublicCircuitPublicInputs,
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
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
        MAX_READ_REQUESTS_PER_CALL,
        NUM_FIELDS_PER_SHA256,
        RETURN_VALUES_LENGTH,
    },
    contrakt::{
        storage_read::StorageRead,
        storage_update_request::StorageUpdateRequest,
    },
    hash::hash_args,
    header::Header,
    utils::reader::Reader,
};

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

    // Header of a block whose state is used during public execution. Set by sequencer to be a header of a block
    // previous to the one in which the tx is included.
    historical_header: Header,
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

            historical_header: inputs.historical_header,
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

    pub fn coinbase(self) -> EthAddress {
        self.inputs.public_global_variables.coinbase
    }

    pub fn fee_recipient(self) -> AztecAddress {
        self.inputs.public_global_variables.fee_recipient
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
            historical_header: self.inputs.historical_header,
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
        let nullifier = process_l1_to_l2_message(self.historical_header.state.l1_to_l2_message_tree.root, this, self.this_portal_address(), self.chain_id(), self.version(), msg_key, content, secret);

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
