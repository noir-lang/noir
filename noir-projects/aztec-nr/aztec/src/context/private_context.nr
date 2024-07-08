use crate::{
    context::{inputs::PrivateContextInputs, packed_returns::PackedReturns},
    messaging::process_l1_to_l2_message, hash::{hash_args_array, ArgsHasher},
    keys::constants::{NULLIFIER_INDEX, OUTGOING_INDEX, NUM_KEY_TYPES, sk_generators},
    oracle::{
    key_validation_request::get_key_validation_request, arguments, returns::pack_returns,
    call_private_function::call_private_function_internal, header::get_header_at,
    logs::{emit_encrypted_note_log, emit_encrypted_event_log},
    enqueue_public_function_call::{
    enqueue_public_function_call_internal, set_public_teardown_function_call_internal,
    parse_public_call_stack_item_from_oracle
}
}
};
use dep::protocol_types::{
    abis::{
    caller_context::CallerContext, function_selector::FunctionSelector,
    max_block_number::MaxBlockNumber,
    validation_requests::{KeyValidationRequest, KeyValidationRequestAndGenerator},
    private_call_request::PrivateCallRequest, private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_call_stack_item::PublicCallStackItem, read_request::ReadRequest, note_hash::NoteHash,
    nullifier::Nullifier, log_hash::{LogHash, NoteLogHash, EncryptedLogHash}
},
    address::{AztecAddress, EthAddress},
    constants::{
    MAX_NOTE_HASHES_PER_CALL, MAX_L2_TO_L1_MSGS_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_ENCRYPTED_LOGS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL,
    MAX_NOTE_ENCRYPTED_LOGS_PER_CALL
},
    header::Header, messaging::l2_to_l1_message::L2ToL1Message, utils::reader::Reader, traits::Empty
};

// When finished, one can call .finish() to convert back to the abi
struct PrivateContext {
    // docs:start:private-context
    inputs: PrivateContextInputs,
    side_effect_counter: u32,

    min_revertible_side_effect_counter: u32,
    is_fee_payer: bool,

    args_hash: Field,
    return_hash: Field,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: BoundedVec<ReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    key_validation_requests_and_generators: BoundedVec<KeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_CALL>,

    note_hashes: BoundedVec<NoteHash, MAX_NOTE_HASHES_PER_CALL>,
    nullifiers: BoundedVec<Nullifier, MAX_NULLIFIERS_PER_CALL>,

    private_call_requests : BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    public_teardown_function_hash: Field,
    l2_to_l1_msgs : BoundedVec<L2ToL1Message, MAX_L2_TO_L1_MSGS_PER_CALL>,
    // docs:end:private-context

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    note_encrypted_logs_hashes: BoundedVec<NoteLogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_CALL>,
    encrypted_logs_hashes: BoundedVec<EncryptedLogHash, MAX_ENCRYPTED_LOGS_PER_CALL>,
    unencrypted_logs_hashes: BoundedVec<LogHash, MAX_UNENCRYPTED_LOGS_PER_CALL>,

    // Contains the last key validation request for each key type. This is used to cache the last request and avoid
    // fetching the same request multiple times.
    // The index of the array corresponds to the key type (0 nullifier, 1 incoming, 2 outgoing, 3 tagging).
    last_key_validation_requests: [Option<KeyValidationRequest>; NUM_KEY_TYPES],
}

impl PrivateContext {
    pub fn new(inputs: PrivateContextInputs, args_hash: Field) -> PrivateContext {
        PrivateContext {
            inputs,
            side_effect_counter: inputs.start_side_effect_counter + 1,
            min_revertible_side_effect_counter: 0,
            is_fee_payer: false,
            args_hash,
            return_hash: 0,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            key_validation_requests_and_generators: BoundedVec::new(),
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            historical_header: inputs.historical_header,
            private_call_requests: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            last_key_validation_requests: [Option::none(); NUM_KEY_TYPES]
        }
    }

    fn msg_sender(self) -> AztecAddress {
        self.inputs.call_context.msg_sender
    }

    fn this_address(self) -> AztecAddress {
        self.inputs.call_context.storage_contract_address
    }

    fn chain_id(self) -> Field {
        self.inputs.tx_context.chain_id
    }

    fn version(self) -> Field {
        self.inputs.tx_context.version
    }

    fn selector(self) -> FunctionSelector {
        self.inputs.call_context.function_selector
    }

    fn get_args_hash(self) -> Field {
        self.args_hash
    }

    fn push_note_hash(&mut self, note_hash: Field) {
        self.note_hashes.push(NoteHash { value: note_hash, counter: self.next_counter() });
    }

    // TODO(#7112): This function is called with non-zero note hash only in 1 of 25 cases in aztec-packages repo
    // - consider creating a separate function with 1 arg for the zero note hash case.
    fn push_nullifier(&mut self, nullifier: Field, nullified_note_hash: Field) {
        self.nullifiers.push(Nullifier { value: nullifier, note_hash: nullified_note_hash, counter: self.next_counter() });
    }

    // Returns the header of a block whose state is used during private execution (not the block the transaction is
    // included in).
    fn get_header(self) -> Header {
        self.historical_header
    }

    // Returns the header of an arbitrary block whose block number is less than or equal to the block number
    // of historical header.
    pub fn get_header_at(self, block_number: u32) -> Header {
        get_header_at(block_number, self)
    }

    pub fn set_return_hash(&mut self, returns_hasher: ArgsHasher) {
        pack_returns(returns_hasher.fields);
        self.return_hash = returns_hasher.hash();
    }

    pub fn finish(self) -> PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.inputs.call_context,
            args_hash: self.args_hash,
            returns_hash: self.return_hash,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            is_fee_payer: self.is_fee_payer,
            max_block_number: self.max_block_number,
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            key_validation_requests_and_generators: self.key_validation_requests_and_generators.storage,
            note_hashes: self.note_hashes.storage,
            nullifiers: self.nullifiers.storage,
            private_call_requests: self.private_call_requests.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            public_teardown_function_hash: self.public_teardown_function_hash,
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage,
            start_side_effect_counter: self.inputs.start_side_effect_counter,
            end_side_effect_counter: self.side_effect_counter,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            historical_header: self.historical_header,
            tx_context: self.inputs.tx_context
        }
    }

    pub fn set_as_fee_payer(&mut self) {
        dep::protocol_types::debug_log::debug_log_format("Setting {0} as fee payer", [self.this_address().to_field()]);
        self.is_fee_payer = true;
    }

    pub fn end_setup(&mut self) {
        // dep::protocol_types::debug_log::debug_log_format(
        //     "Ending setup at counter {0}",
        //     [self.side_effect_counter as Field]
        // );
        self.min_revertible_side_effect_counter = self.side_effect_counter;
    }

    // docs:start:max-block-number
    pub fn set_tx_max_block_number(&mut self, max_block_number: u32) {
        // docs:end:max-block-number
        self.max_block_number = MaxBlockNumber::min_with_u32(self.max_block_number, max_block_number);
    }

    pub fn push_note_hash_read_request(&mut self, note_hash: Field) {
        let side_effect = ReadRequest { value: note_hash, counter: self.next_counter() };
        self.note_hash_read_requests.push(side_effect);
    }

    pub fn push_nullifier_read_request(&mut self, nullifier: Field) {
        let request = ReadRequest { value: nullifier, counter: self.next_counter() };
        self.nullifier_read_requests.push(request);
    }

    pub fn request_nsk_app(&mut self, npk_m_hash: Field) -> Field {
        self.request_sk_app(npk_m_hash, NULLIFIER_INDEX)
    }

    pub fn request_ovsk_app(&mut self, ovpk_m_hash: Field) -> Field {
        self.request_sk_app(ovpk_m_hash, OUTGOING_INDEX)
    }

    fn request_sk_app(&mut self, pk_m_hash: Field, key_index: Field) -> Field {
        let cached_request = self.last_key_validation_requests[key_index].unwrap_or(KeyValidationRequest::empty());

        if cached_request.pk_m.hash() == pk_m_hash {
            // We get a match so the cached request is the latest one 
            cached_request.sk_app
        } else {
            // We didn't get a match meaning the cached result is stale. We fetch new values from oracle and instruct
            // protocol circuits to validate them by storing the validation request in context.
            let request = get_key_validation_request(pk_m_hash, key_index);
            let request_and_generator = KeyValidationRequestAndGenerator { request, sk_app_generator: sk_generators[key_index] };
            // We constrain that the pk_m_hash matches the one in the request (otherwise we could get an arbitrary
            // valid key request and not the one corresponding to pk_m_hash).
            assert(request.pk_m.hash() == pk_m_hash);
            self.key_validation_requests_and_generators.push(request_and_generator);
            self.last_key_validation_requests[key_index] = Option::some(request);
            request.sk_app
        }
    }

    // docs:start:context_message_portal
    pub fn message_portal(&mut self, recipient: EthAddress, content: Field) {
        // docs:end:context_message_portal
        let message = L2ToL1Message { recipient, content, counter: self.next_counter() };
        self.l2_to_l1_msgs.push(message);
    }

    // docs:start:context_consume_l1_to_l2_message
    // docs:start:consume_l1_to_l2_message
    pub fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress) {
        // docs:end:context_consume_l1_to_l2_message
        let nullifier = process_l1_to_l2_message(
            self.historical_header.state.l1_to_l2_message_tree.root,
            self.this_address(),
            sender,
            self.chain_id(),
            self.version(),
            content,
            secret
        );

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_nullifier(nullifier, 0)
    }
    // docs:end:consume_l1_to_l2_message

    // NB: A randomness value of 0 signals that the kernels should not mask the contract address
    // used in siloing later on e.g. 'handshaking' contract w/ known address.
    pub fn emit_raw_event_log_with_masked_address<M>(&mut self, randomness: Field, log: [u8; M], log_hash: Field) {
        let counter = self.next_counter();
        let contract_address = self.this_address();
        let len = log.len() as Field + 4;
        let side_effect = EncryptedLogHash { value: log_hash, counter, length: len, randomness };
        self.encrypted_logs_hashes.push(side_effect);

        emit_encrypted_event_log(contract_address, randomness, log, counter);
    }

    pub fn emit_raw_note_log<M>(&mut self, note_hash_counter: u32, log: [u8; M], log_hash: Field) {
        let counter = self.next_counter();
        let len = log.len() as Field + 4;
        let side_effect = NoteLogHash { value: log_hash, counter, length: len, note_hash_counter };
        self.note_encrypted_logs_hashes.push(side_effect);

        emit_encrypted_note_log(note_hash_counter, log, counter);
    }

    pub fn call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn static_call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, true, false)
    }

    pub fn delegate_call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, false, true)
    }

    pub fn call_private_function_no_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, false, false)
    }

    pub fn static_call_private_function_no_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, true, false)
    }

    pub fn delegate_call_private_function_no_args<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, false, true)
    }

    pub fn call_private_function_with_packed_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) -> PackedReturns {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let start_side_effect_counter = self.side_effect_counter;
        let item = call_private_function_internal(
            contract_address,
            function_selector,
            args_hash,
            start_side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        assert_eq(item.public_inputs.start_side_effect_counter, start_side_effect_counter);
        let end_side_effect_counter = item.public_inputs.end_side_effect_counter;
        self.side_effect_counter = end_side_effect_counter + 1;

        // TODO (fees) figure out why this crashes the prover and enable it
        // we need this in order to pay fees inside child call contexts
        // assert(
        //     (item.public_inputs.min_revertible_side_effect_counter == 0 as u32)
        //     | (item.public_inputs.min_revertible_side_effect_counter
        //         > self.min_revertible_side_effect_counter)
        // );

        // if item.public_inputs.min_revertible_side_effect_counter
        //     > self.min_revertible_side_effect_counter {
        //     self.min_revertible_side_effect_counter = item.public_inputs.min_revertible_side_effect_counter;
        // }

        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the call generated by the oracle matches our request.
        assert(item.public_inputs.call_context.is_delegate_call == is_delegate_call);
        assert(item.public_inputs.call_context.is_static_call == is_static_call);

        if (is_delegate_call) {
            // For delegate calls, we also constrain the execution context address for the nested call to be equal to our address.
            assert(
                item.public_inputs.call_context.storage_contract_address.eq(self.inputs.call_context.storage_contract_address)
            );
            assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.msg_sender));
        } else {
            // For non-delegate calls, we also constrain the execution context address for the nested call to be equal to the address we called.
            assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));
            assert(
                item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address)
            );
        }

        let mut caller_context = CallerContext::empty();
        caller_context.is_static_call = self.inputs.call_context.is_static_call;
        if is_delegate_call {
            caller_context.msg_sender = self.inputs.call_context.msg_sender;
            caller_context.storage_contract_address = self.inputs.call_context.storage_contract_address;
        }
        self.private_call_requests.push(
            PrivateCallRequest {
            target: item.contract_address,
            call_context: item.public_inputs.call_context,
            function_data: item.function_data,
            args_hash: item.public_inputs.args_hash,
            returns_hash: item.public_inputs.returns_hash,
            caller_context,
            start_side_effect_counter,
            end_side_effect_counter
        }
        );

        PackedReturns::new(item.public_inputs.returns_hash)
    }

    pub fn call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn static_call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, true, false)
    }

    pub fn delegate_call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, true)
    }

    pub fn call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, false)
    }

    pub fn static_call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, true, false)
    }

    pub fn delegate_call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, true)
    }

    pub fn call_public_function_with_packed_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let fields = enqueue_public_function_call_internal(
            contract_address,
            function_selector,
            args_hash,
            self.side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        let item = parse_public_call_stack_item_from_oracle(fields);
        self.validate_call_stack_item_from_oracle(
            item,
            contract_address,
            function_selector,
            args_hash,
            is_static_call,
            is_delegate_call
        );

        self.side_effect_counter = self.side_effect_counter + 1;
        self.public_call_stack_hashes.push(item.get_compressed().hash());
    }

    pub fn set_public_teardown_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.set_public_teardown_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn set_public_teardown_function_with_packed_args<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let fields = set_public_teardown_function_call_internal(
            contract_address,
            function_selector,
            args_hash,
            self.side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        let item = parse_public_call_stack_item_from_oracle(fields);
        self.validate_call_stack_item_from_oracle(
            item,
            contract_address,
            function_selector,
            args_hash,
            is_static_call,
            is_delegate_call
        );

        self.side_effect_counter = self.side_effect_counter + 1;
        self.public_teardown_function_hash = item.get_compressed().hash();
    }

    fn validate_call_stack_item_from_oracle(
        self,
        item: PublicCallStackItem,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert_eq(item.public_inputs.start_side_effect_counter, self.side_effect_counter);

        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        assert(item.public_inputs.call_context.is_delegate_call == is_delegate_call);
        assert(item.public_inputs.call_context.is_static_call == is_static_call);

        if (is_delegate_call) {
            // For delegate calls, we also constrain the execution context address for the nested call to be equal to our address.
            assert(
                item.public_inputs.call_context.storage_contract_address.eq(self.inputs.call_context.storage_contract_address)
            );
            assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.msg_sender));
        } else {
            // For non-delegate calls, we also constrain the execution context address for the nested call to be equal to the address we called.
            assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));
            assert(
                item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address)
            );
        }
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.side_effect_counter;
        self.side_effect_counter += 1;
        counter
    }
}

impl Empty for PrivateContext {
    fn empty() -> Self {
        PrivateContext {
            inputs: PrivateContextInputs::empty(),
            side_effect_counter: 0 as u32,
            min_revertible_side_effect_counter: 0 as u32,
            is_fee_payer: false,
            args_hash: 0,
            return_hash: 0,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            key_validation_requests_and_generators: BoundedVec::new(),
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            private_call_requests: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            l2_to_l1_msgs: BoundedVec::new(),
            historical_header: Header::empty(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            last_key_validation_requests: [Option::none(); NUM_KEY_TYPES]
        }
    }
}
