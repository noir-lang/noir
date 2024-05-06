use crate::{
    context::{
    inputs::PublicContextInputs, interface::ContextInterface, interface::PublicContextInterface,
    gas::GasOpts
},
    messaging::process_l1_to_l2_message,
    oracle::{arguments, public_call::call_public_function_internal, returns, logs_traits::ToBytesForUnencryptedLog},
    hash::{hash_args, ArgsHasher, compute_unencrypted_log_hash}
};
use dep::protocol_types::{
    abis::{
    global_variables::GlobalVariables, function_selector::FunctionSelector,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_call_stack_item::PublicCallStackItem,
    public_circuit_public_inputs::PublicCircuitPublicInputs, read_request::ReadRequest,
    note_hash::NoteHash, nullifier::Nullifier, side_effect::SideEffect
},
    hash::silo_nullifier, address::{AztecAddress, EthAddress},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_UNENCRYPTED_LOGS_PER_CALL
},
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest}, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, utils::reader::Reader, traits::{Deserialize, Empty}
};

struct PublicContext {
    inputs: PublicContextInputs,
    side_effect_counter: u32,

    args_hash : Field,
    return_hash : Field,

    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    nullifier_non_existent_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL>,
    contract_storage_update_requests: BoundedVec<StorageUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>,
    contract_storage_reads: BoundedVec<StorageRead, MAX_PUBLIC_DATA_READS_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,

    new_note_hashes: BoundedVec<NoteHash, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<Nullifier, MAX_NEW_NULLIFIERS_PER_CALL>,

    new_l2_to_l1_msgs: BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    unencrypted_logs_hashes: BoundedVec<SideEffect, MAX_UNENCRYPTED_LOGS_PER_CALL>,
    unencrypted_log_preimages_length: Field,

    // Header of a block whose state is used during public execution. Set by sequencer to be a header of a block
    // previous to the one in which the tx is included.
    historical_header: Header,
    prover_address: AztecAddress,
}

impl PublicContext {
    pub fn new(inputs: PublicContextInputs, args_hash: Field) -> PublicContext {
        PublicContext {
            inputs,
            side_effect_counter: inputs.start_side_effect_counter,
            args_hash,
            return_hash: 0,
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            contract_storage_update_requests: BoundedVec::new(),
            contract_storage_reads: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            new_l2_to_l1_msgs: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            unencrypted_log_preimages_length: 0,
            historical_header: inputs.historical_header,
            prover_address: AztecAddress::zero()
        }
    }

    pub fn call_public_function_no_args<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> FunctionReturns<RETURNS_COUNT> {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, false)
    }

    pub fn static_call_public_function_no_args<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> FunctionReturns<RETURNS_COUNT> {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, true, false)
    }

    pub fn delegate_call_public_function_no_args<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> FunctionReturns<RETURNS_COUNT> {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, true)
    }

    pub fn call_public_function_with_packed_args<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) -> FunctionReturns<RETURNS_COUNT> {
        let side_effect_counter = self.side_effect_counter;
        // TODO(6052): get next value from output of `call_public_function_internal`
        self.side_effect_counter += 1;

        let raw_returns = call_public_function_internal(
            contract_address,
            function_selector,
            args_hash,
            side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        FunctionReturns::new(raw_returns)
    }

    pub fn set_return_hash(&mut self, returns_hasher: ArgsHasher) {
        returns::pack_returns(returns_hasher.fields);
        self.return_hash = returns_hasher.hash();
    }

    // Keep private or ask the AVM team if you want to change it.
    fn push_nullifier_read_request(&mut self, nullifier: Field) {
        let request = ReadRequest { value: nullifier, counter: self.side_effect_counter };
        self.nullifier_read_requests.push(request);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    // Keep private or ask the AVM team if you want to change it.
    fn push_nullifier_non_existent_read_request(&mut self, nullifier: Field) {
        let request = ReadRequest { value: nullifier, counter: self.side_effect_counter };
        self.nullifier_non_existent_read_requests.push(request);
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    pub fn finish(self) -> PublicCircuitPublicInputs {
        // Compute the public call stack hashes
        let pub_circuit_pub_inputs = PublicCircuitPublicInputs {
            call_context: self.inputs.call_context, // Done
            args_hash: self.args_hash, // Done
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            contract_storage_update_requests: self.contract_storage_update_requests.storage,
            contract_storage_reads: self.contract_storage_reads.storage,
            returns_hash: self.return_hash,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.inputs.start_side_effect_counter,
            end_side_effect_counter: self.side_effect_counter,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length + 4,
            historical_header: self.inputs.historical_header,
            global_variables: self.inputs.public_global_variables,
            prover_address: self.prover_address,
            revert_code: 0,
            start_gas_left: self.inputs.gas_left,
            end_gas_left: self.inputs.gas_left, // AVM should decrease this value
            transaction_fee: self.inputs.transaction_fee
        };
        pub_circuit_pub_inputs
    }
}

impl ContextInterface for PublicContext {
    fn msg_sender(self) -> AztecAddress {
        self.inputs.call_context.msg_sender
    }

    fn this_address(self) -> AztecAddress {
        self.inputs.call_context.storage_contract_address
    }

    fn chain_id(self) -> Field {
        self.inputs.public_global_variables.chain_id
    }

    fn version(self) -> Field {
        self.inputs.public_global_variables.version
    }

    fn selector(self) -> FunctionSelector {
        self.inputs.call_context.function_selector
    }

    fn get_args_hash(self) -> Field {
        self.args_hash
    }

    fn push_new_note_hash(&mut self, note_hash: Field) {
        self.new_note_hashes.push(NoteHash { value: note_hash, counter: self.side_effect_counter });
        self.side_effect_counter = self.side_effect_counter + 1;
    }

    fn push_new_nullifier(&mut self, nullifier: Field, _nullified_note_hash: Field) {
        self.new_nullifiers.push(Nullifier {
            value: nullifier,
            note_hash: 0, // cannot nullify pending notes in public context
            counter: self.side_effect_counter
        });
        self.side_effect_counter = self.side_effect_counter + 1;
    }
}

impl PublicContextInterface for PublicContext {
    fn block_number(self) -> Field {
        self.inputs.public_global_variables.block_number
    }

    fn timestamp(self) -> u64 {
        self.inputs.public_global_variables.timestamp
    }

    fn coinbase(self) -> EthAddress {
        self.inputs.public_global_variables.coinbase
    }

    fn fee_recipient(self) -> AztecAddress {
        self.inputs.public_global_variables.fee_recipient
    }

    fn fee_per_da_gas(self) -> Field {
        self.inputs.public_global_variables.gas_fees.fee_per_da_gas
    }

    fn fee_per_l2_gas(self) -> Field {
        self.inputs.public_global_variables.gas_fees.fee_per_l2_gas
    }

    fn transaction_fee(self) -> Field {
        self.inputs.transaction_fee
    }

    fn nullifier_exists(self, unsiloed_nullifier: Field, address: AztecAddress) -> bool {
        // Current public can only check for settled nullifiers, so we always silo.
        let siloed_nullifier = silo_nullifier(address, unsiloed_nullifier);
        nullifier_exists_oracle(siloed_nullifier) == 1
    }

    fn message_portal(&mut self, recipient: EthAddress, content: Field) {
        let message = L2ToL1Message { recipient, content };
        self.new_l2_to_l1_msgs.push(message);
    }

    // We can consume message with a secret in public context because the message cannot be modified and therefore
    // there is no front-running risk (e.g. somebody could front run you to claim your tokens to your address).
    // Leaf index is not used in public context, but it is used in the AVMContext which will replace it.
    fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress, _leaf_index: Field) {
        let this = (*self).this_address();
        let nullifier = process_l1_to_l2_message(
            self.historical_header.state.l1_to_l2_message_tree.root,
            this,
            sender,
            self.chain_id(),
            self.version(),
            content,
            secret
        );

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, 0)
    }

    fn emit_unencrypted_log<T, N, M>(&mut self, log: T) where T: ToBytesForUnencryptedLog<N, M> {
        let event_selector = 5; // TODO: compute actual event selector.
        let contract_address = self.this_address();
        let log_slice = log.to_be_bytes_arr();
        let log_hash = compute_unencrypted_log_hash(
            contract_address,
            event_selector,
            log
        );
        let side_effect = SideEffect { value: log_hash, counter: self.side_effect_counter };
        self.unencrypted_logs_hashes.push(side_effect);
        self.side_effect_counter = self.side_effect_counter + 1;
        // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
        self.unencrypted_log_preimages_length = self.unencrypted_log_preimages_length + 44 + log_slice.len().to_field();
        // Call oracle to broadcast log
        let _void = emit_unencrypted_log_oracle(contract_address, event_selector, log, side_effect.counter);
    }

    fn call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        _gas: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    fn static_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        _gas: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, true, false)
    }

    fn delegate_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field]
    ) -> FunctionReturns<RETURNS_COUNT> {
        let args_hash = hash_args(args);
        assert(args_hash == arguments::pack_arguments(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, true)
    }
}

impl Empty for PublicContext {
    fn empty() -> Self {
        PublicContext {
            inputs: PublicContextInputs::empty(),
            side_effect_counter: 0 as u32,
            args_hash : 0,
            return_hash : 0,
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            contract_storage_update_requests: BoundedVec::new(),
            contract_storage_reads: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            new_l2_to_l1_msgs: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            unencrypted_log_preimages_length: 0,
            historical_header: Header::empty(),
            prover_address: AztecAddress::zero(),
        }
    }
}

#[oracle(checkNullifierExists)]
fn nullifier_exists_oracle(nullifier: Field) -> Field {}

#[oracle(emitUnencryptedLog)]
fn emit_unencrypted_log_oracle<T>(
    _contract_address: AztecAddress,
    _event_selector: Field,
    _message: T,
    _counter: u32
) -> Field {}

struct FunctionReturns<N> {
    values: [Field; N]
}

impl<N> FunctionReturns<N> {
    pub fn new(values: [Field; N]) -> FunctionReturns<N> {
        FunctionReturns { values }
    }

    pub fn assert_empty(returns: FunctionReturns<0>) {
        assert(returns.values.len() == 0);
    }

    pub fn raw(self) -> [Field; N] {
        self.values
    }

    pub fn deserialize_into<T>(self) -> T where T: Deserialize<N> {
        Deserialize::deserialize(self.raw())
    }
}
