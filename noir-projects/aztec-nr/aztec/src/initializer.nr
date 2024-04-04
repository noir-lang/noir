use dep::protocol_types::{
    address::AztecAddress, hash::{silo_nullifier, pedersen_hash},
    constants::GENERATOR_INDEX__CONSTRUCTOR, abis::function_selector::FunctionSelector
};

use crate::{
    context::{PrivateContext, PublicContext, AvmContext, ContextInterface},
    oracle::get_contract_instance::get_contract_instance,
    oracle::get_contract_instance::get_contract_instance_avm,
    history::nullifier_inclusion::prove_nullifier_inclusion
};

pub fn mark_as_initialized_public(context: &mut PublicContext) {
    mark_as_initialized(context);
}

pub fn mark_as_initialized_avm(context: &mut AvmContext) {
    mark_as_initialized(context);
}

pub fn mark_as_initialized_private(context: &mut PrivateContext) {
    mark_as_initialized(context);
}

fn mark_as_initialized<TContext>(context: &mut TContext) where TContext: ContextInterface {
    let init_nullifier = compute_unsiloed_contract_initialization_nullifier((*context).this_address());
    ContextInterface::push_new_nullifier(context, init_nullifier, 0);
}

pub fn assert_is_initialized_public(context: &mut PublicContext) {
    let init_nullifier = compute_contract_initialization_nullifier(context.this_address());
    prove_nullifier_inclusion(init_nullifier, *context);
}

pub fn assert_is_initialized_avm(context: &mut AvmContext) {
    // WARNING: the AVM always expects UNSILOED nullifiers!
    // TODO(fcarreiro@): Change current private/public to take unsiloed nullifiers and an address.
    let init_nullifier = compute_unsiloed_contract_initialization_nullifier(context.this_address());
    assert(context.nullifier_exists(init_nullifier));
}

pub fn assert_is_initialized_private(context: &mut PrivateContext) {
    let init_nullifier = compute_contract_initialization_nullifier(context.this_address());
    prove_nullifier_inclusion(init_nullifier, *context);
}

fn compute_contract_initialization_nullifier(address: AztecAddress) -> Field {
    silo_nullifier(
        address,
        compute_unsiloed_contract_initialization_nullifier(address)
    )
}

fn compute_unsiloed_contract_initialization_nullifier(address: AztecAddress) -> Field {
    address.to_field()
}

pub fn assert_initialization_matches_address_preimage_public(context: PublicContext) {
    assert_initialization_matches_address_preimage(context);
}

pub fn assert_initialization_matches_address_preimage_avm(context: AvmContext) {
    let address = context.this_address();
    let instance = get_contract_instance_avm(address).unwrap();
    let expected_init = compute_initialization_hash(context.selector(), context.get_args_hash());
    assert(instance.initialization_hash == expected_init, "Initialization hash does not match");
    assert(
        (instance.deployer.is_zero()) | (instance.deployer == context.msg_sender()), "Initializer address is not the contract deployer"
    );
}

pub fn assert_initialization_matches_address_preimage_private(context: PrivateContext) {
    assert_initialization_matches_address_preimage(context);
}

fn assert_initialization_matches_address_preimage<TContext>(context: TContext) where TContext: ContextInterface {
    let address = context.this_address();
    let instance = get_contract_instance(address);
    let expected_init = compute_initialization_hash(context.selector(), context.get_args_hash());
    assert(instance.initialization_hash == expected_init, "Initialization hash does not match");
    assert(
        (instance.deployer.is_zero()) | (instance.deployer == context.msg_sender()), "Initializer address is not the contract deployer"
    );
}

pub fn compute_initialization_hash(init_selector: FunctionSelector, init_args_hash: Field) -> Field {
    pedersen_hash(
        [init_selector.to_field(), init_args_hash],
        GENERATOR_INDEX__CONSTRUCTOR
    )
}
