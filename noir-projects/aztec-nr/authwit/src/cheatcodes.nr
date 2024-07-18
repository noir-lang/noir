use dep::aztec::{
    protocol_types::address::AztecAddress,
    context::{public_context::PublicContext, call_interfaces::CallInterface}, test::helpers::cheatcodes,
    oracle::execution::{get_block_number, get_contract_address}, hash::hash_args
};

use crate::auth::{compute_inner_authwit_hash, compute_authwit_message_hash, set_authorized};

pub fn add_private_authwit_from_call_interface<C, M, T, P, Env>(
    on_behalf_of: AztecAddress,
    caller: AztecAddress,
    call_interface: C
) where C: CallInterface<M, T, P, Env> {
    let target = call_interface.get_contract_address();
    let inputs = cheatcodes::get_private_context_inputs(get_block_number());
    let chain_id = inputs.tx_context.chain_id;
    let version = inputs.tx_context.version;
    let args_hash = hash_args(call_interface.get_args());
    let selector = call_interface.get_selector();
    let inner_hash = compute_inner_authwit_hash([caller.to_field(), selector.to_field(), args_hash]);
    let message_hash = compute_authwit_message_hash(target, chain_id, version, inner_hash);
    cheatcodes::add_authwit(on_behalf_of, message_hash);
}

pub fn add_public_authwit_from_call_interface<C, M, T, P, Env>(
    on_behalf_of: AztecAddress,
    caller: AztecAddress,
    call_interface: C
) where C: CallInterface<M, T, P, Env> {
    let current_contract = get_contract_address();
    cheatcodes::set_contract_address(on_behalf_of);
    let target = call_interface.get_contract_address();
    let inputs = cheatcodes::get_private_context_inputs(get_block_number());
    let chain_id = inputs.tx_context.chain_id;
    let version = inputs.tx_context.version;
    let args_hash = hash_args(call_interface.get_args());
    let selector = call_interface.get_selector();
    let inner_hash = compute_inner_authwit_hash([caller.to_field(), selector.to_field(), args_hash]);
    let message_hash = compute_authwit_message_hash(target, chain_id, version, inner_hash);
    let mut inputs = cheatcodes::get_public_context_inputs();
    let mut context = PublicContext::new(inputs);
    set_authorized(&mut context, message_hash, true);
    cheatcodes::set_contract_address(current_contract);
}
