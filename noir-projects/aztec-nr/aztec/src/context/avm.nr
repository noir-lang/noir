use dep::protocol_types::address::{AztecAddress, EthAddress};

// Getters that will be converted by the transpiler into their
// own opcodes
struct AVMContext {}

impl AVMContext {
    // Empty new function enables retaining context.<value> syntax
    pub fn new() -> Self {
        Self {}
    }

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

    #[oracle(emitNoteHash)]
    pub fn emit_note_hash(self, note_hash: Field) {}

    #[oracle(nullifierExists)]
    pub fn check_nullifier_exists(self, nullifier: Field) -> u8 {}

    #[oracle(emitNullifier)]
    pub fn emit_nullifier(self, nullifier: Field) {}

    // The functions below allow interface-equivalence with PrivateContext
    // for emitting note hashes and nullifiers
    pub fn push_new_note_hash(self: &mut Self, note_hash: Field) {
        self.emit_note_hash(note_hash);
    }
    pub fn push_new_nullifier(self: &mut Self, nullifier: Field, _nullified_commitment: Field) {
        // Cannot nullify pending commitments in AVM, so `nullified_commitment` is not used
        self.emit_nullifier(nullifier);
    }
}
