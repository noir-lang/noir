use crate::hash::{compute_secret_hash, compute_message_hash, compute_message_nullifier};
use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{L1_TO_L2_MESSAGE_LENGTH, NESTED_CALL_L2_GAS_BUFFER}, header::Header
};
use dep::protocol_types::traits::Serialize;
use dep::protocol_types::abis::function_selector::FunctionSelector;
use dep::protocol_types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs;
use crate::context::inputs::avm_context_inputs::AvmContextInputs;
use crate::context::interface::ContextInterface;
use crate::context::interface::PublicContextInterface;
use crate::context::gas::GasOpts;
use crate::context::public_context::FunctionReturns;

struct AvmContext {
    inputs: AvmContextInputs,
}

impl AvmContext {
    pub fn new(inputs: AvmContextInputs) -> Self {
        AvmContext { inputs }
    }

    pub fn storage_address(self) -> AztecAddress {
        storage_address()
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
    pub fn emit_unencrypted_log<T>(&mut self, event_selector: Field, log: T) {
        emit_unencrypted_log(event_selector, log);
    }
    pub fn note_hash_exists(self, note_hash: Field, leaf_index: Field) -> bool {
        note_hash_exists(note_hash, leaf_index) == 1
    }
    pub fn l1_to_l2_msg_exists(self, msg_hash: Field, msg_leaf_index: Field) -> bool {
        l1_to_l2_msg_exists(msg_hash, msg_leaf_index) == 1
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

    fn fee_per_da_gas(self) -> Field {
        assert(false, "'fee_per_da_gas' not implemented!");
        0
    }

    fn fee_per_l2_gas(self) -> Field {
        assert(false, "'fee_per_l2_gas' not implemented!");
        0
    }

    fn transaction_fee(self) -> Field {
        transaction_fee()
    }

    fn nullifier_exists(self, unsiloed_nullifier: Field, address: AztecAddress) -> bool {
        nullifier_exists(unsiloed_nullifier, address.to_field()) == 1
    }

    fn emit_unencrypted_log<T,N,M>(&mut self, log: T) {
        let event_selector = 5;  // Matches current PublicContext.
        self.emit_unencrypted_log(event_selector, log);
    }

    fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress, leaf_index: Field) {
        let secret_hash = compute_secret_hash(secret);
        let message_hash = compute_message_hash(
            sender,
            self.chain_id(),
            /*recipient=*/self.this_address(),
            self.version(),
            content,
            secret_hash
        );
        let nullifier = compute_message_nullifier(message_hash, secret, leaf_index);

        assert(!self.nullifier_exists(nullifier, self.this_address()), "L1-to-L2 message is already nullified");
        assert(self.l1_to_l2_msg_exists(message_hash, leaf_index), "Tried to consume nonexistent L1-to-L2 message");
        
        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, 0);
    }

    fn message_portal(&mut self, recipient: EthAddress, content: Field) {
        send_l2_to_l1_msg(recipient, content);
    }

    fn call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        temporary_function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {        
        let results = call(
            gas_for_call(gas_opts),
            contract_address,
            args,
            temporary_function_selector.to_field()
        );
        let data_to_return: [Field; RETURNS_COUNT] = results.0;
        let success: u8 = results.1;
        assert(success == 1, "Nested call failed!");

        FunctionReturns::new(data_to_return)
    }

    fn static_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        temporary_function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {
        let (data_to_return, success): ([Field; RETURNS_COUNT], u8) = call_static(
            gas_for_call(gas_opts),
            contract_address,
            args,
            temporary_function_selector.to_field()
        );

        assert(success == 1, "Nested static call failed!");
        FunctionReturns::new(data_to_return)
    }

    fn delegate_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field]
    ) -> FunctionReturns<RETURNS_COUNT> {
        assert(false, "'delegate_call_public_function' not implemented!");
        FunctionReturns::new([0; RETURNS_COUNT])
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
    fn chain_id(self) -> Field {
        chain_id()
    }
    fn version(self) -> Field {
        version()
    }
    fn selector(self) -> FunctionSelector {
        FunctionSelector::from_field(self.inputs.selector)
    }
    fn get_args_hash(self) -> Field {
        self.inputs.args_hash
    }
}

// Helper functions
fn gas_for_call(user_gas: GasOpts) -> [Field; 2] {
    [
        user_gas.l2_gas.unwrap_or_else(|| l2_gas_left() - NESTED_CALL_L2_GAS_BUFFER),
        user_gas.da_gas.unwrap_or_else(|| da_gas_left())
    ]
}

// AVM oracles (opcodes) follow, do not use directly.
#[oracle(avmOpcodeAddress)]
fn address() -> AztecAddress {}

#[oracle(avmOpcodeStorageAddress)]
fn storage_address() -> AztecAddress {}

#[oracle(avmOpcodeSender)]
fn sender() -> AztecAddress {}

#[oracle(avmOpcodePortal)]
fn portal() -> EthAddress {}

#[oracle(avmOpcodeFeePerL2Gas)]
fn fee_per_l2_gas() -> Field {}

#[oracle(avmOpcodeFeePerDaGas)]
fn fee_per_da_gas() -> Field {}

#[oracle(avmOpcodeTransactionFee)]
fn transaction_fee() -> Field {}

#[oracle(avmOpcodeChainId)]
fn chain_id() -> Field {}

#[oracle(avmOpcodeVersion)]
fn version() -> Field {}

#[oracle(avmOpcodeBlockNumber)]
fn block_number() -> Field {}

#[oracle(avmOpcodeTimestamp)]
fn timestamp() -> u64 {}

#[oracle(avmOpcodeL2GasLeft)]
fn l2_gas_left() -> Field {}

#[oracle(avmOpcodeDaGasLeft)]
fn da_gas_left() -> Field {}

#[oracle(avmOpcodeNoteHashExists)]
fn note_hash_exists(note_hash: Field, leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeEmitNoteHash)]
fn emit_note_hash(note_hash: Field) {}

#[oracle(avmOpcodeNullifierExists)]
fn nullifier_exists(nullifier: Field, address: Field) -> u8 {}

#[oracle(avmOpcodeEmitNullifier)]
fn emit_nullifier(nullifier: Field) {}

#[oracle(amvOpcodeEmitUnencryptedLog)]
fn emit_unencrypted_log<T>(event_selector: Field, message: T) {}

#[oracle(avmOpcodeL1ToL2MsgExists)]
fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeSendL2ToL1Msg)]
fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {}

#[oracle(avmOpcodeCall)]
fn call<RET_SIZE>(
    gas: [Field; 2], // gas allocation: [l2_gas, da_gas]
    address: AztecAddress,
    args: [Field],
    // TODO(5110): consider passing in calldata directly
    temporary_function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success

#[oracle(avmOpcodeStaticCall)]
fn call_static<RET_SIZE>(
    gas: [Field; 2], // gas allocation: [l2_gas, da_gas]
    address: AztecAddress,
    args: [Field],
    // TODO(5110): consider passing in calldata directly
    temporary_function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success
