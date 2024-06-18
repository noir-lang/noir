use dep::protocol_types::{abis::function_selector::FunctionSelector, address::{AztecAddress, PartialAddress}};
use crate::context::inputs::{PublicContextInputs, PrivateContextInputs};
use crate::test::helpers::types::{Deployer, TestAccount};
use crate::keys::public_keys::PublicKeys;

unconstrained pub fn reset() {
    oracle_reset();
}

unconstrained pub fn get_contract_address() -> AztecAddress {
    oracle_get_contract_address()
}

unconstrained pub fn set_contract_address(address: AztecAddress) {
    oracle_set_contract_address(address);
}

unconstrained pub fn get_block_number() -> u32 {
    oracle_get_block_number()
}

unconstrained pub fn advance_blocks(blocks: u32) {
    oracle_time_travel(blocks);
}

unconstrained pub fn get_private_context_inputs(historical_block_number: u32) -> PrivateContextInputs {
    oracle_get_private_context_inputs(historical_block_number)
}

unconstrained pub fn get_public_context_inputs() -> PublicContextInputs {
    oracle_get_public_context_inputs()
}

unconstrained pub fn deploy<N, M>(
    path: str<N>,
    initializer: str<M>,
    args: [Field],
    public_keys_hash: Field
) -> AztecAddress {
    oracle_deploy(path, initializer, args, public_keys_hash)
}

unconstrained pub fn direct_storage_write<N>(
    contract_address: AztecAddress,
    storage_slot: Field,
    fields: [Field; N]
) {
    let _hash = direct_storage_write_oracle(contract_address, storage_slot, fields);
}

unconstrained pub fn create_account() -> TestAccount {
    oracle_create_account()
}

unconstrained pub fn add_account(secret: Field, partial_address: PartialAddress) -> TestAccount {
    oracle_add_account(secret, partial_address)
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

#[oracle(reset)]
fn oracle_reset() {}

#[oracle(getContractAddress)] 
fn oracle_get_contract_address() -> AztecAddress {}

#[oracle(setContractAddress)]
fn oracle_set_contract_address(address: AztecAddress) {}

#[oracle(getBlockNumber)]
fn oracle_get_block_number() -> u32 {}

#[oracle(timeTravel)]
fn oracle_time_travel(blocks: u32) {}

#[oracle(getPrivateContextInputs)]
fn oracle_get_private_context_inputs(historical_block_number: u32) -> PrivateContextInputs {}

#[oracle(getPublicContextInputs)]
fn oracle_get_public_context_inputs() -> PublicContextInputs {}

#[oracle(deploy)]
fn oracle_deploy<N, M>(
    path: str<N>,
    initializer: str<M>,
    args: [Field],
    public_keys_hash: Field
) -> AztecAddress {}

#[oracle(directStorageWrite)]
fn direct_storage_write_oracle<N>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _values: [Field; N]
) -> [Field; N] {}

#[oracle(createAccount)]
fn oracle_create_account() -> TestAccount {}

#[oracle(addAccount)]
fn oracle_add_account(secret: Field, partial_address: PartialAddress) -> TestAccount {}

#[oracle(deriveKeys)]
fn oracle_derive_keys(secret: Field) -> PublicKeys {}

#[oracle(getMsgSender)]
fn oracle_get_msg_sender() -> AztecAddress {}

#[oracle(setMsgSender)]
fn oracle_set_msg_sender(msg_sender: AztecAddress) {}

#[oracle(getSideEffectsCounter)]
fn oracle_get_side_effects_counter() -> u32 {}
