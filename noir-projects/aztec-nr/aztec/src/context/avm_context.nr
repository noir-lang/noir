use dep::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH, header::Header};
use dep::protocol_types::traits::Serialize;
use dep::protocol_types::abis::function_selector::FunctionSelector;
use dep::protocol_types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs;
use dep::protocol_types::constants::RETURN_VALUES_LENGTH;
use crate::context::inputs::avm_context_inputs::AvmContextInputs;
use crate::context::interface::ContextInterface;
use crate::context::interface::PublicContextInterface;

struct AvmContext {
    inputs: AvmContextInputs,
}

impl AvmContext {
    pub fn new(inputs: AvmContextInputs) -> Self {
        AvmContext { inputs }
    }

    pub fn origin(self) -> AztecAddress {
        origin()
    }
    pub fn storage_address(self) -> AztecAddress {
        storage_address()
    }
    pub fn fee_per_l1_gas(self) -> Field {
        fee_per_l1_gas()
    }
    pub fn fee_per_l2_gas(self) -> Field {
        fee_per_l2_gas()
    }
    pub fn fee_per_da_gas(self) -> Field {
        fee_per_da_gas()
    }
    /**
    * Emit a log with the given event selector and message.
    *
    * @param event_selector The event selector for the log.
    * @param message The message to emit in the log.
    * Should be automatically convertible to [Field; N]. For example str<N> works with
    * one char per field. Otherwise you can use CompressedString.
    */
    pub fn accumulate_unencrypted_logs<T>(&mut self, event_selector: Field, log: T) {
        emit_unencrypted_log(event_selector, log);
    }
    pub fn note_hash_exists(self, note_hash: Field, leaf_index: Field) -> bool {
        note_hash_exists(note_hash, leaf_index) == 1
    }
    pub fn l1_to_l2_msg_exists(self, msg_hash: Field, msg_leaf_index: Field) -> bool {
        l1_to_l2_msg_exists(msg_hash, msg_leaf_index) == 1
    }
    pub fn nullifier_exists(self, nullifier: Field) -> bool {
        nullifier_exists(nullifier) == 1
    }

    fn call_public_function_raw<ARGS_COUNT, RET_COUNT>(
        self: &mut Self,
        gas: [Field; 3],
        contract_address: AztecAddress,
        temporary_function_selector: Field,
        args: [Field; ARGS_COUNT]
    ) -> ([Field; RET_COUNT], u8) {
        call(gas, contract_address, args, temporary_function_selector)
    }

    fn static_call_public_function_raw<ARGS_COUNT, RET_COUNT>(
        self: &mut Self,
        gas: [Field; 3],
        contract_address: AztecAddress,
        temporary_function_selector: Field,
        args: [Field; ARGS_COUNT]
    ) -> ([Field; RET_COUNT], u8) {
        call_static(gas, contract_address, args, temporary_function_selector)
    }
}

impl PublicContextInterface for AvmContext {
    fn block_number(self) -> Field {
        block_number()
    }

    fn timestamp(self) -> u64 {
        timestamp()
    }

    fn coinbase(self) -> EthAddress {
        assert(false, "'coinbase' not implemented!");
        EthAddress::zero()
    }

    fn fee_recipient(self) -> AztecAddress {
        assert(false, "'fee_recipient' not implemented!");
        AztecAddress::zero()
    }

    fn push_nullifier_read_request(&mut self, nullifier: Field) {
        assert(false, "'push_nullifier_read_request' not implemented!");
    }

    fn push_nullifier_non_existent_read_request(&mut self, nullifier: Field) {
        assert(false, "'push_nullifier_non_existent_read_request' not implemented!");
    }

    fn accumulate_encrypted_logs<N>(&mut self, log: [Field; N]) {
        assert(false, "'accumulate_encrypted_logs' not implemented!");
    }

    fn accumulate_unencrypted_logs<T>(&mut self, log: T) {
        let event_selector = 0;
        self.accumulate_unencrypted_logs(event_selector, log);
    }

    fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress) {
        assert(false, "'consume_l1_to_l2_message' not implemented!");
    }

    fn message_portal(&mut self, recipient: EthAddress, content: Field) {
        send_l2_to_l1_msg(recipient, content);
    }

    fn call_public_function<ARGS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        temporary_function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let gas = [/*l1_gas*/10000, /*l2_gas*/10000, /*da_gas*/10000];

        let results = call(
            gas,
            contract_address,
            args,
            temporary_function_selector.to_field()
        );
        let data_to_return: [Field; RETURN_VALUES_LENGTH] = results.0;
        let success: u8 = results.1;
        assert(success == 1, "Nested call failed!");

        data_to_return
    }

    fn static_call_public_function<ARGS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        temporary_function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> [Field; RETURN_VALUES_LENGTH] {
        let gas = [/*l1_gas*/10000, /*l2_gas*/10000, /*da_gas*/10000];

        let (data_to_return, success): ([Field; RETURN_VALUES_LENGTH], u8) = call_static(
            gas,
            contract_address,
            args,
            temporary_function_selector.to_field()
        );

        assert(success == 1, "Nested static call failed!");
        data_to_return
    }

    fn delegate_call_public_function<ARGS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> [Field; RETURN_VALUES_LENGTH] {
        assert(false, "'delegate_call_public_function' not implemented!");
        [0; RETURN_VALUES_LENGTH]
    }
}

impl ContextInterface for AvmContext {
    fn push_new_note_hash(&mut self, note_hash: Field) {
        emit_note_hash(note_hash);
    }
    fn push_new_nullifier(&mut self, nullifier: Field, _nullified_commitment: Field) {
        // Cannot nullify pending commitments in AVM, so `nullified_commitment` is not used
        emit_nullifier(nullifier);
    }
    fn msg_sender(self) -> AztecAddress {
        sender()
    }
    fn this_address(self) -> AztecAddress {
        address()
    }
    fn this_portal_address(self) -> EthAddress {
        portal()
    }
    fn chain_id(self) -> Field {
        chain_id()
    }
    fn version(self) -> Field {
        version()
    }
    fn selector(self) -> FunctionSelector {
        FunctionSelector::from_field(self.inputs.selector)
    }
    fn get_header(self) -> Header {
        assert(false, "'get_header' not implemented!");
        Header::empty()
    }
    fn get_args_hash(self) -> Field {
        self.inputs.args_hash
    }
}

// AVM oracles (opcodes) follow, do not use directly.
#[oracle(avmOpcodeAddress)]
fn address() -> AztecAddress {}

#[oracle(avmOpcodeStorageAddress)]
fn storage_address() -> AztecAddress {}

#[oracle(avmOpcodeOrigin)]
fn origin() -> AztecAddress {}

#[oracle(avmOpcodeSender)]
fn sender() -> AztecAddress {}

#[oracle(avmOpcodePortal)]
fn portal() -> EthAddress {}

#[oracle(avmOpcodeFeePerL1Gas)]
fn fee_per_l1_gas() -> Field {}

#[oracle(avmOpcodeFeePerL2Gas)]
fn fee_per_l2_gas() -> Field {}

#[oracle(avmOpcodeFeePerDaGas)]
fn fee_per_da_gas() -> Field {}

#[oracle(avmOpcodeChainId)]
fn chain_id() -> Field {}

#[oracle(avmOpcodeVersion)]
fn version() -> Field {}

#[oracle(avmOpcodeBlockNumber)]
fn block_number() -> Field {}

#[oracle(avmOpcodeTimestamp)]
fn timestamp() -> u64 {}

#[oracle(avmOpcodeNoteHashExists)]
fn note_hash_exists(note_hash: Field, leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeEmitNoteHash)]
fn emit_note_hash(note_hash: Field) {}

#[oracle(avmOpcodeNullifierExists)]
fn nullifier_exists(nullifier: Field) -> u8 {}

#[oracle(avmOpcodeEmitNullifier)]
fn emit_nullifier(nullifier: Field) {}

#[oracle(amvOpcodeEmitUnencryptedLog)]
fn emit_unencrypted_log<T>(event_selector: Field, message: T) {}

#[oracle(avmOpcodeL1ToL2MsgExists)]
fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeSendL2ToL1Msg)]
fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {}

#[oracle(avmOpcodeCall)]
fn call<ARGS_COUNT, RET_SIZE>(
    gas: [Field; 3], // gas allocation: [l1_gas, l2_gas, da_gas]
    address: AztecAddress,
    args: [Field; ARGS_COUNT],
    // TODO(5110): consider passing in calldata directly
    temporary_function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success

#[oracle(avmOpcodeStaticCall)]
fn call_static<ARGS_COUNT, RET_SIZE>(
    gas: [Field; 3], // gas allocation: [l1_gas, l2_gas, da_gas]
    address: AztecAddress,
    args: [Field; ARGS_COUNT],
    // TODO(5110): consider passing in calldata directly
    temporary_function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success