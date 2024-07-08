use dep::protocol_types::{
    abis::{
    function_selector::FunctionSelector, public_call_stack_item::PublicCallStackItem,
    function_data::FunctionData, public_circuit_public_inputs::PublicCircuitPublicInputs,
    call_context::CallContext, read_request::ReadRequest, note_hash::NoteHash, nullifier::Nullifier,
    log_hash::LogHash, global_variables::GlobalVariables, gas::Gas
},
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    messaging::l2_to_l1_message::L2ToL1Message, header::Header, address::AztecAddress,
    utils::reader::Reader,
    constants::{
    MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_NOTE_HASHES_PER_CALL, MAX_L2_TO_L1_MSGS_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL,
    ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH
}
};

#[oracle(enqueuePublicFunctionCall)]
unconstrained fn enqueue_public_function_call_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH] {}

unconstrained pub fn enqueue_public_function_call_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH] {
    enqueue_public_function_call_oracle(
        contract_address,
        function_selector,
        args_hash,
        side_effect_counter,
        is_static_call,
        is_delegate_call
    )
}

#[oracle(setPublicTeardownFunctionCall)]
unconstrained fn set_public_teardown_function_call_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH] {}

unconstrained pub fn set_public_teardown_function_call_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH] {
    set_public_teardown_function_call_oracle(
        contract_address,
        function_selector,
        args_hash,
        side_effect_counter,
        is_static_call,
        is_delegate_call
    )
}

pub fn parse_public_call_stack_item_from_oracle(fields: [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH]) -> PublicCallStackItem {
    let mut reader = Reader::new(fields);

    // Note: Not using PublicCirclePublicInputs::deserialize here, because everything below args_hash is 0 and
    // there is no more data in fields because there is only ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE fields!
    // WARNING: if updating, see comment in public_call_stack_item.ts's PublicCallStackItem.hash()
    let item = PublicCallStackItem {
        contract_address: AztecAddress::from_field(reader.read()),
        function_data: FunctionData { selector: FunctionSelector::from_field(reader.read()), is_private: false },
        public_inputs: PublicCircuitPublicInputs {
            call_context: reader.read_struct(CallContext::deserialize),
            start_side_effect_counter: reader.read_u32(),
            args_hash: reader.read(),
            returns_hash: 0,
            note_hash_read_requests: [ReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
            nullifier_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
            nullifier_non_existent_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL],
            l1_to_l2_msg_read_requests: [ReadRequest::empty(); MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL],
            contract_storage_update_requests: [StorageUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
            contract_storage_reads: [StorageRead::empty(); MAX_PUBLIC_DATA_READS_PER_CALL],
            public_call_stack_hashes: [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
            note_hashes: [NoteHash::empty(); MAX_NOTE_HASHES_PER_CALL],
            nullifiers: [Nullifier::empty(); MAX_NULLIFIERS_PER_CALL],
            l2_to_l1_msgs: [L2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_CALL],
            end_side_effect_counter: 0,
            unencrypted_logs_hashes: [LogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL],
            historical_header: Header::empty(),
            global_variables: GlobalVariables::empty(),
            prover_address: AztecAddress::zero(),
            revert_code: 0,
            start_gas_left: Gas::empty(),
            end_gas_left: Gas::empty(),
            transaction_fee: 0
        },
        is_execution_request: true
    };
    reader.finish();

    item
}
