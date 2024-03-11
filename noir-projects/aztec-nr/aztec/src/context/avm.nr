use dep::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH};
use dep::protocol_types::traits::{Serialize};
use dep::protocol_types::abis::function_selector::FunctionSelector;

// Getters that will be converted by the transpiler into their
// own opcodes
struct AVMContext {}

impl AVMContext {
    // Empty new function enables retaining context.<value> syntax
    pub fn new() -> Self {
        Self {}
    }

    // OPCODES
    #[oracle(avmOpcodeAddress)]
    pub fn address(self) -> AztecAddress {}

    #[oracle(avmOpcodeStorageAddress)]
    pub fn storage_address(self) -> AztecAddress {}

    #[oracle(avmOpcodeOrigin)]
    pub fn origin(self) -> AztecAddress {}

    #[oracle(avmOpcodeSender)]
    pub fn sender(self) -> AztecAddress {}

    #[oracle(avmOpcodePortal)]
    pub fn portal(self) -> EthAddress {}

    #[oracle(avmOpcodeFeePerL1Gas)]
    pub fn fee_per_l1_gas(self) -> Field {}

    #[oracle(avmOpcodeFeePerL2Gas)]
    pub fn fee_per_l2_gas(self) -> Field {}

    #[oracle(avmOpcodeFeePerDaGas)]
    pub fn fee_per_da_gas(self) -> Field {}

    #[oracle(avmOpcodeChainId)]
    pub fn chain_id(self) -> Field {}

    #[oracle(avmOpcodeVersion)]
    pub fn version(self) -> Field {}

    #[oracle(avmOpcodeBlockNumber)]
    pub fn block_number(self) -> Field {}

    #[oracle(avmOpcodeTimestamp)]
    pub fn timestamp(self) -> Field {}

    // #[oracle(avmOpcodeContractCallDepth)]
    // pub fn contract_call_depth(self) -> Field {}

    #[oracle(avmOpcodeNoteHashExists)]
    pub fn note_hash_exists(self, note_hash: Field, leaf_index: Field) -> u8 {}

    #[oracle(avmOpcodeEmitNoteHash)]
    pub fn emit_note_hash(self, note_hash: Field) {}

    #[oracle(avmOpcodeNullifierExists)]
    pub fn nullifier_exists(self, nullifier: Field) -> u8 {}

    #[oracle(avmOpcodeEmitNullifier)]
    pub fn emit_nullifier(self, nullifier: Field) {}

    /**
     * Emit a log with the given event selector and message.
     *
     * @param event_selector The event selector for the log.
     * @param message The message to emit in the log.
     * Should be automatically convertible to [Field; N]. For example str<N> works with
     * one char per field. Otherwise you can use CompressedString.
     */
    #[oracle(amvOpcodeEmitUnencryptedLog)]
    pub fn emit_unencrypted_log<T>(self, event_selector: Field, message: T) {}

    #[oracle(avmOpcodeL1ToL2MsgExists)]
    pub fn l1_to_l2_msg_exists(self, msg_hash: Field, msg_leaf_index: Field) -> u8 {}

    #[oracle(avmOpcodeSendL2ToL1Msg)]
    pub fn send_l2_to_l1_msg(self, recipient: EthAddress, content: Field) {}

    #[oracle(avmOpcodeCall)]
    fn call<ARGS_COUNT, RET_SIZE>(
        self,
        gas: [Field; 3], // gas allocation: [l1Gas, l2Gas, daGas]
        address: AztecAddress,
        args: [Field; ARGS_COUNT],
        // TODO(5110): consider passing in calldata directly
        temporary_function_selector: Field
    ) -> ([Field; RET_SIZE], u8) {}
    //    ^ return data      ^ success

    ////////////////////////////////////////////////////////////////////////////////
    // The functions below allow interface-equivalence with current public/private
    ////////////////////////////////////////////////////////////////////////////////
    pub fn this_address(self) -> AztecAddress {
        self.address()
    }

    #[oracle(avmOpcodeSendL2ToL1Msg)]
    pub fn message_portal(&mut self, recipient: EthAddress, content: Field) {}

    pub fn consume_l1_to_l2_message(
        &mut self,
        _msg_key: Field,
        _content: Field,
        _secret: Field,
        _sender: EthAddress
    ) {
        assert(false, "Not implemented!");
    }

    #[oracle(avmOpcodeEmitNoteHash)]
    pub fn push_new_note_hash(self: &mut Self, note_hash: Field) {}

    pub fn push_new_nullifier(self: &mut Self, nullifier: Field, _nullified_commitment: Field) {
        // Cannot nullify pending commitments in AVM, so `nullified_commitment` is not used
        self.emit_nullifier(nullifier);
    }

    pub fn call_public_function<ARGS_COUNT, RET_SIZE>(
        self: Self,
        contract_address: AztecAddress,
        temporary_function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> [Field; RET_SIZE] {
        let gas = [/*l1Gas*/42, /*l2Gas*/24, /*daGas*/420];

        let results = self.call(
            gas,
            contract_address,
            args,
            temporary_function_selector.to_field()
        );
        let returnData: [Field; RET_SIZE] = results.0;
        let success: u8 = results.1;
        assert(success == 1, "Nested call failed!");

        returnData
    }
}
