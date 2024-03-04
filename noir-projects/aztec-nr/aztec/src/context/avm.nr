use dep::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH};

// Getters that will be converted by the transpiler into their
// own opcodes
struct AVMContext {}

impl AVMContext {
    // Empty new function enables retaining context.<value> syntax
    pub fn new() -> Self {
        Self {}
    }

    // OPCODES
    #[oracle(address)]
    pub fn address(self) -> AztecAddress {}

    #[oracle(storageAddress)]
    pub fn storage_address(self) -> AztecAddress {}

    #[oracle(origin)]
    pub fn origin(self) -> AztecAddress {}

    #[oracle(sender)]
    pub fn sender(self) -> AztecAddress {}

    #[oracle(portal)]
    pub fn portal(self) -> EthAddress {}

    #[oracle(feePerL1Gas)]
    pub fn fee_per_l1_gas(self) -> Field {}

    #[oracle(feePerL2Gas)]
    pub fn fee_per_l2_gas(self) -> Field {}

    #[oracle(feePerDaGas)]
    pub fn fee_per_da_gas(self) -> Field {}

    #[oracle(chainId)]
    pub fn chain_id(self) -> Field {}

    #[oracle(version)]
    pub fn version(self) -> Field {}

    #[oracle(blockNumber)]
    pub fn block_number(self) -> Field {}

    #[oracle(timestamp)]
    pub fn timestamp(self) -> Field {}

    // #[oracle(contractCallDepth)]
    // pub fn contract_call_depth(self) -> Field {}

    #[oracle(avmOpcodeNoteHashExists)]
    pub fn note_hash_exists(self, note_hash: Field, leaf_index: Field) -> u8 {}

    #[oracle(emitNoteHash)]
    pub fn emit_note_hash(self, note_hash: Field) {}

    #[oracle(nullifierExists)]
    pub fn nullifier_exists(self, nullifier: Field) -> u8 {}

    #[oracle(emitNullifier)]
    pub fn emit_nullifier(self, nullifier: Field) {}

    #[oracle(l1ToL2MsgExists)]
    pub fn l1_to_l2_msg_exists(self, msg_hash: Field, msg_leaf_index: Field) -> u8 {}

    #[oracle(sendL2ToL1Msg)]
    pub fn send_l2_to_l1_msg(self, recipient: EthAddress, content: Field) {}

    ///////////////////////////////////////////////////////////////////////////
    // The functions below allow interface-equivalence with PrivateContext
    ///////////////////////////////////////////////////////////////////////////
    pub fn this_address(self) -> AztecAddress {
        self.address()
    }

    #[oracle(sendL2ToL1Msg)]
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

    #[oracle(emitNoteHash)]
    pub fn push_new_note_hash(self: &mut Self, note_hash: Field) {}

    pub fn push_new_nullifier(self: &mut Self, nullifier: Field, _nullified_commitment: Field) {
        // Cannot nullify pending commitments in AVM, so `nullified_commitment` is not used
        self.emit_nullifier(nullifier);
    }
}
