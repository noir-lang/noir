use dep::protocol_types::{
    abis::function_selector::FunctionSelector, address::{AztecAddress, PartialAddress},
    constants::CONTRACT_INSTANCE_LENGTH, contract_instance::ContractInstance
};
use crate::context::inputs::{PublicContextInputs, PrivateContextInputs};
use crate::test::helpers::utils::{Deployer, TestAccount};
use crate::keys::public_keys::PublicKeys;

unconstrained pub fn reset() {
    oracle_reset();
}

unconstrained pub fn set_contract_address(address: AztecAddress) {
    oracle_set_contract_address(address);
}

unconstrained pub fn advance_blocks_by(blocks: u32) {
    oracle_advance_blocks_by(blocks);
}

unconstrained pub fn get_private_context_inputs(historical_block_number: u32) -> PrivateContextInputs {
    oracle_get_private_context_inputs(historical_block_number)
}

unconstrained pub fn get_public_context_inputs() -> PublicContextInputs {
    oracle_get_public_context_inputs()
}

unconstrained pub fn deploy<let N: u32, let M: u32, let P: u32>(
    path: str<N>,
    name: str<M>,
    initializer: str<P>,
    args: [Field],
    public_keys_hash: Field
) -> ContractInstance {
    let instance_fields = oracle_deploy(path, name, initializer, args, public_keys_hash);
    ContractInstance::deserialize(instance_fields)
}

unconstrained pub fn direct_storage_write<let N: u32>(contract_address: AztecAddress, storage_slot: Field, fields: [Field; N]) {
    let _hash = direct_storage_write_oracle(contract_address, storage_slot, fields);
}

unconstrained pub fn create_account() -> TestAccount {
    oracle_create_account()
}

unconstrained pub fn add_account(secret: Field) -> TestAccount {
    oracle_add_account(secret)
}

unconstrained pub fn derive_keys(secret: Field) -> PublicKeys {
    oracle_derive_keys(secret)
}

unconstrained pub fn set_msg_sender(msg_sender: AztecAddress) {
    oracle_set_msg_sender(msg_sender)
}

unconstrained pub fn get_msg_sender() -> AztecAddress {
    oracle_get_msg_sender()
}

unconstrained pub fn get_side_effects_counter() -> u32 {
    oracle_get_side_effects_counter()
}

unconstrained pub fn add_authwit(address: AztecAddress, message_hash: Field) {
    orable_add_authwit(address, message_hash)
}

unconstrained pub fn assert_public_call_fails(target_address: AztecAddress, function_selector: FunctionSelector, args: [Field]) {
    oracle_assert_public_call_fails(target_address, function_selector, args)
}

unconstrained pub fn assert_private_call_fails(
    target_address: AztecAddress,
    function_selector: FunctionSelector,
    argsHash: Field,
    sideEffectsCounter: Field,
    isStaticCall: bool,
    isDelegateCall: bool
) {
    oracle_assert_private_call_fails(
        target_address,
        function_selector,
        argsHash,
        sideEffectsCounter,
        isStaticCall,
        isDelegateCall
    )
}

unconstrained pub fn add_nullifiers(contractAddress: AztecAddress, nullifiers: [Field]) {
    oracle_add_nullifiers(contractAddress, nullifiers)
}

unconstrained pub fn add_note_hashes(contractAddress: AztecAddress, inner_note_hashes: [Field]) {
    oracle_add_note_hashes(contractAddress, inner_note_hashes)
}

unconstrained pub fn get_function_selector() -> FunctionSelector {
    oracle_get_function_selector()
}

unconstrained pub fn set_fn_selector(selector: FunctionSelector) {
    oracle_set_function_selector(selector)
}

#[oracle(reset)]
unconstrained fn oracle_reset() {}

#[oracle(setContractAddress)]
unconstrained fn oracle_set_contract_address(address: AztecAddress) {}

#[oracle(advanceBlocksBy)]
unconstrained fn oracle_advance_blocks_by(blocks: u32) {}

#[oracle(getPrivateContextInputs)]
unconstrained fn oracle_get_private_context_inputs(historical_block_number: u32) -> PrivateContextInputs {}

#[oracle(getPublicContextInputs)]
unconstrained fn oracle_get_public_context_inputs() -> PublicContextInputs {}

#[oracle(deploy)]
unconstrained fn oracle_deploy<let N: u32, let M: u32, let P: u32>(
    path: str<N>,
    name: str<M>,
    initializer: str<P>,
    args: [Field],
    public_keys_hash: Field
) -> [Field; CONTRACT_INSTANCE_LENGTH] {}

#[oracle(directStorageWrite)]
unconstrained fn direct_storage_write_oracle<let N: u32>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _values: [Field; N]
) -> [Field; N] {}

#[oracle(createAccount)]
unconstrained fn oracle_create_account() -> TestAccount {}

#[oracle(addAccount)]
unconstrained fn oracle_add_account(secret: Field) -> TestAccount {}

#[oracle(deriveKeys)]
unconstrained fn oracle_derive_keys(secret: Field) -> PublicKeys {}

#[oracle(getMsgSender)]
unconstrained fn oracle_get_msg_sender() -> AztecAddress {}

#[oracle(setMsgSender)]
unconstrained fn oracle_set_msg_sender(msg_sender: AztecAddress) {}

#[oracle(getSideEffectsCounter)]
unconstrained fn oracle_get_side_effects_counter() -> u32 {}

#[oracle(addAuthWitness)]
unconstrained fn orable_add_authwit(address: AztecAddress, message_hash: Field) {}

#[oracle(assertPublicCallFails)]
unconstrained fn oracle_assert_public_call_fails(
    target_address: AztecAddress,
    function_selector: FunctionSelector,
    args: [Field]
) {}

#[oracle(assertPrivateCallFails)]
unconstrained fn oracle_assert_private_call_fails(
    target_address: AztecAddress,
    function_selector: FunctionSelector,
    argsHash: Field,
    sideEffectsCounter: Field,
    isStaticCall: bool,
    isDelegateCall: bool
) {}

#[oracle(addNullifiers)]
unconstrained fn oracle_add_nullifiers(contractAddress: AztecAddress, nullifiers: [Field]) {}

#[oracle(addNoteHashes)]
unconstrained fn oracle_add_note_hashes(contractAddress: AztecAddress, inner_note_hashes: [Field]) {}

#[oracle(getFunctionSelector)]
unconstrained fn oracle_get_function_selector() -> FunctionSelector {}

#[oracle(setFunctionSelector)]
unconstrained fn oracle_set_function_selector(selector: FunctionSelector) {}
