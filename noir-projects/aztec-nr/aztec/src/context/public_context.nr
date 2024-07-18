use crate::hash::{compute_secret_hash, compute_message_hash, compute_message_nullifier};
use dep::protocol_types::address::{AztecAddress, EthAddress};
use dep::protocol_types::constants::MAX_FIELD_VALUE;
use dep::protocol_types::traits::{Serialize, Deserialize, Empty};
use dep::protocol_types::abis::function_selector::FunctionSelector;
use crate::context::inputs::public_context_inputs::PublicContextInputs;
use crate::context::gas::GasOpts;

struct PublicContext {
    inputs: PublicContextInputs,
}

impl PublicContext {
    pub fn new(inputs: PublicContextInputs) -> Self {
        PublicContext { inputs }
    }

    pub fn emit_unencrypted_log<T, let N: u32>(_self: &mut Self, log: T) where T: Serialize<N> {
        emit_unencrypted_log(Serialize::serialize(log).as_slice());
    }

    pub fn note_hash_exists(_self: Self, note_hash: Field, leaf_index: Field) -> bool {
        note_hash_exists(note_hash, leaf_index) == 1
    }

    pub fn l1_to_l2_msg_exists(_self: Self, msg_hash: Field, msg_leaf_index: Field) -> bool {
        l1_to_l2_msg_exists(msg_hash, msg_leaf_index) == 1
    }

    fn nullifier_exists(_self: Self, unsiloed_nullifier: Field, address: AztecAddress) -> bool {
        nullifier_exists(unsiloed_nullifier, address.to_field()) == 1
    }

    fn consume_l1_to_l2_message(
        &mut self,
        content: Field,
        secret: Field,
        sender: EthAddress,
        leaf_index: Field
    ) {
        let secret_hash = compute_secret_hash(secret);
        let message_hash = compute_message_hash(
            sender,
            self.chain_id(),
            /*recipient=*/ self.this_address(),
            self.version(),
            content,
            secret_hash
        );
        let nullifier = compute_message_nullifier(message_hash, secret, leaf_index);

        assert(
            !self.nullifier_exists(nullifier, self.this_address()), "L1-to-L2 message is already nullified"
        );
        assert(
            self.l1_to_l2_msg_exists(message_hash, leaf_index), "Tried to consume nonexistent L1-to-L2 message"
        );

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_nullifier(nullifier, 0);
    }

    fn message_portal(_self: &mut Self, recipient: EthAddress, content: Field) {
        send_l2_to_l1_msg(recipient, content);
    }

    fn call_public_function<let RETURNS_COUNT: u32>(
        _self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {
        let results = call(
            gas_for_call(gas_opts),
            contract_address,
            args,
            function_selector.to_field()
        );
        let data_to_return: [Field; RETURNS_COUNT] = results.0;
        let success: u8 = results.1;
        assert(success == 1, "Nested call failed!");

        FunctionReturns::new(data_to_return)
    }

    fn static_call_public_function<let RETURNS_COUNT: u32>(
        _self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT> {
        let (data_to_return, success): ([Field; RETURNS_COUNT], u8) = call_static(
            gas_for_call(gas_opts),
            contract_address,
            args,
            function_selector.to_field()
        );

        assert(success == 1, "Nested static call failed!");
        FunctionReturns::new(data_to_return)
    }

    fn delegate_call_public_function<let RETURNS_COUNT: u32>(
        _self: &mut Self,
        _contract_address: AztecAddress,
        _function_selector: FunctionSelector,
        _args: [Field]
    ) -> FunctionReturns<RETURNS_COUNT> {
        assert(false, "'delegate_call_public_function' not implemented!");
        FunctionReturns::new([0; RETURNS_COUNT])
    }

    fn push_note_hash(_self: &mut Self, note_hash: Field) {
        emit_note_hash(note_hash);
    }
    fn push_nullifier(_self: &mut Self, nullifier: Field, _nullified_commitment: Field) {
        // Cannot nullify pending commitments in AVM, so `nullified_commitment` is not used
        emit_nullifier(nullifier);
    }

    fn this_address(_self: Self) -> AztecAddress {
        address()
    }
    pub fn storage_address(_self: Self) -> AztecAddress {
        storage_address()
    }
    fn msg_sender(_self: Self) -> AztecAddress {
        sender()
    }
    fn selector(_self: Self) -> FunctionSelector {
        FunctionSelector::from_u32(function_selector())
    }
    fn get_args_hash(self) -> Field {
        self.inputs.args_hash
    }
    fn transaction_fee(_self: Self) -> Field {
        transaction_fee()
    }

    fn chain_id(_self: Self) -> Field {
        chain_id()
    }
    fn version(_self: Self) -> Field {
        version()
    }
    fn block_number(_self: Self) -> Field {
        block_number()
    }
    fn timestamp(_self: Self) -> u64 {
        timestamp()
    }
    pub fn fee_per_l2_gas(_self: Self) -> Field {
        fee_per_l2_gas()
    }
    pub fn fee_per_da_gas(_self: Self) -> Field {
        fee_per_da_gas()
    }

    fn l2_gas_left(_self: Self) -> Field {
        l2_gas_left()
    }
    fn da_gas_left(_self: Self) -> Field {
        da_gas_left()
    }

    fn raw_storage_read<let N: u32>(_self: Self, storage_slot: Field) -> [Field; N] {
        storage_read(storage_slot)
    }

    fn storage_read<T, let N: u32>(self, storage_slot: Field) -> T where T: Deserialize<N> {
        T::deserialize(self.raw_storage_read(storage_slot))
    }

    fn raw_storage_write<let N: u32>(_self: Self, storage_slot: Field, values: [Field; N]) {
        storage_write(storage_slot, values);
    }

    fn storage_write<T, let N: u32>(self, storage_slot: Field, value: T) where T: Serialize<N> {
        self.raw_storage_write(storage_slot, value.serialize());
    }
}

// Helper functions
fn gas_for_call(user_gas: GasOpts) -> [Field; 2] {
    // It's ok to use the max possible gas here, because the gas will be
    // capped by the gas left in the (STATIC)CALL instruction.
    [
        user_gas.l2_gas.unwrap_or(MAX_FIELD_VALUE),
        user_gas.da_gas.unwrap_or(MAX_FIELD_VALUE)
    ]
}

// Unconstrained opcode wrappers (do not use directly).
// TODO(https://github.com/AztecProtocol/aztec-packages/issues/6420): reconsider.
unconstrained fn address() -> AztecAddress {
    address_opcode()
}
unconstrained fn storage_address() -> AztecAddress {
    storage_address_opcode()
}
unconstrained fn sender() -> AztecAddress {
    sender_opcode()
}
unconstrained fn portal() -> EthAddress {
    portal_opcode()
}
unconstrained fn function_selector() -> u32 {
    function_selector_opcode()
}
unconstrained fn transaction_fee() -> Field {
    transaction_fee_opcode()
}
unconstrained fn chain_id() -> Field {
    chain_id_opcode()
}
unconstrained fn version() -> Field {
    version_opcode()
}
unconstrained fn block_number() -> Field {
    block_number_opcode()
}
unconstrained fn timestamp() -> u64 {
    timestamp_opcode()
}
unconstrained fn fee_per_l2_gas() -> Field {
    fee_per_l2_gas_opcode()
}
unconstrained fn fee_per_da_gas() -> Field {
    fee_per_da_gas_opcode()
}
unconstrained fn l2_gas_left() -> Field {
    l2_gas_left_opcode()
}
unconstrained fn da_gas_left() -> Field {
    da_gas_left_opcode()
}
unconstrained fn note_hash_exists(note_hash: Field, leaf_index: Field) -> u8 {
    note_hash_exists_opcode(note_hash, leaf_index)
}
unconstrained fn emit_note_hash(note_hash: Field) {
    emit_note_hash_opcode(note_hash)
}
unconstrained fn nullifier_exists(nullifier: Field, address: Field) -> u8 {
    nullifier_exists_opcode(nullifier, address)
}
unconstrained fn emit_nullifier(nullifier: Field) {
    emit_nullifier_opcode(nullifier)
}
unconstrained fn emit_unencrypted_log(message: [Field]) {
    emit_unencrypted_log_opcode(message)
}
unconstrained fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> u8 {
    l1_to_l2_msg_exists_opcode(msg_hash, msg_leaf_index)
}
unconstrained fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {
    send_l2_to_l1_msg_opcode(recipient, content)
}
unconstrained fn call<let RET_SIZE: u32>(
    gas: [Field; 2],
    address: AztecAddress,
    args: [Field],
    function_selector: Field
) -> ([Field; RET_SIZE], u8) {
    call_opcode(gas, address, args, function_selector)
}
unconstrained fn call_static<let RET_SIZE: u32>(
    gas: [Field; 2],
    address: AztecAddress,
    args: [Field],
    function_selector: Field
) -> ([Field; RET_SIZE], u8) {
    call_static_opcode(gas, address, args, function_selector)
}

unconstrained fn storage_read<let N: u32>(storage_slot: Field) -> [Field; N] {
    storage_read_opcode(storage_slot, N as Field)
}

unconstrained fn storage_write<let N: u32>(storage_slot: Field, values: [Field; N]) {
    storage_write_opcode(storage_slot, values);
}

impl Empty for PublicContext {
    fn empty() -> Self {
        PublicContext::new(PublicContextInputs::empty())
    }
}

// AVM oracles (opcodes) follow, do not use directly.
#[oracle(avmOpcodeAddress)]
unconstrained fn address_opcode() -> AztecAddress {}

#[oracle(avmOpcodeStorageAddress)]
unconstrained fn storage_address_opcode() -> AztecAddress {}

#[oracle(avmOpcodeSender)]
unconstrained fn sender_opcode() -> AztecAddress {}

#[oracle(avmOpcodePortal)]
unconstrained fn portal_opcode() -> EthAddress {}

#[oracle(avmOpcodeFunctionSelector)]
unconstrained fn function_selector_opcode() -> u32 {}

#[oracle(avmOpcodeTransactionFee)]
unconstrained fn transaction_fee_opcode() -> Field {}

#[oracle(avmOpcodeChainId)]
unconstrained fn chain_id_opcode() -> Field {}

#[oracle(avmOpcodeVersion)]
unconstrained fn version_opcode() -> Field {}

#[oracle(avmOpcodeBlockNumber)]
unconstrained fn block_number_opcode() -> Field {}

#[oracle(avmOpcodeTimestamp)]
unconstrained fn timestamp_opcode() -> u64 {}

#[oracle(avmOpcodeFeePerL2Gas)]
unconstrained fn fee_per_l2_gas_opcode() -> Field {}

#[oracle(avmOpcodeFeePerDaGas)]
unconstrained fn fee_per_da_gas_opcode() -> Field {}

#[oracle(avmOpcodeL2GasLeft)]
unconstrained fn l2_gas_left_opcode() -> Field {}

#[oracle(avmOpcodeDaGasLeft)]
unconstrained fn da_gas_left_opcode() -> Field {}

#[oracle(avmOpcodeNoteHashExists)]
unconstrained fn note_hash_exists_opcode(note_hash: Field, leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeEmitNoteHash)]
unconstrained fn emit_note_hash_opcode(note_hash: Field) {}

#[oracle(avmOpcodeNullifierExists)]
unconstrained fn nullifier_exists_opcode(nullifier: Field, address: Field) -> u8 {}

#[oracle(avmOpcodeEmitNullifier)]
unconstrained fn emit_nullifier_opcode(nullifier: Field) {}

#[oracle(avmOpcodeEmitUnencryptedLog)]
unconstrained fn emit_unencrypted_log_opcode(message: [Field]) {}

#[oracle(avmOpcodeL1ToL2MsgExists)]
unconstrained fn l1_to_l2_msg_exists_opcode(msg_hash: Field, msg_leaf_index: Field) -> u8 {}

#[oracle(avmOpcodeSendL2ToL1Msg)]
unconstrained fn send_l2_to_l1_msg_opcode(recipient: EthAddress, content: Field) {}

#[oracle(avmOpcodeCall)]
unconstrained fn call_opcode<let RET_SIZE: u32>(
    gas: [Field; 2], // gas allocation: [l2_gas, da_gas]
    address: AztecAddress,
    args: [Field],
    // TODO(5110): consider passing in calldata directly
    function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success

#[oracle(avmOpcodeStaticCall)]
unconstrained fn call_static_opcode<let RET_SIZE: u32>(
    gas: [Field; 2], // gas allocation: [l2_gas, da_gas]
    address: AztecAddress,
    args: [Field],
    // TODO(5110): consider passing in calldata directly
    function_selector: Field
) -> ([Field; RET_SIZE], u8) {}
//    ^ return data      ^ success

#[oracle(avmOpcodeStorageRead)]
unconstrained fn storage_read_opcode<let N: u32>(storage_slot: Field, length: Field) -> [Field; N] {}

#[oracle(avmOpcodeStorageWrite)]
unconstrained fn storage_write_opcode<let N: u32>(storage_slot: Field, values: [Field; N]) {}

struct FunctionReturns<let N: u32> {
    values: [Field; N]
}

impl<let N: u32> FunctionReturns<N> {
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
