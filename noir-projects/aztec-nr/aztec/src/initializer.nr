use dep::protocol_types::{
    hash::{silo_nullifier, pedersen_hash}, constants::GENERATOR_INDEX__CONSTRUCTOR,
    abis::function_selector::FunctionSelector
};

use crate::{
    context::{PrivateContext, PublicContext, ContextInterface},
    oracle::get_contract_instance::get_contract_instance,
    history::nullifier_inclusion::prove_nullifier_inclusion
};

pub fn mark_as_initialized<TContext>(context: &mut TContext) where TContext: ContextInterface {
    let init_nullifier = compute_unsiloed_contract_initialization_nullifier(*context);
    ContextInterface::push_new_nullifier(context, init_nullifier, 0);
}

pub fn assert_is_initialized<TContext>(context: &mut TContext) where TContext: ContextInterface {
    let init_nullifier = compute_contract_initialization_nullifier(*context);
    prove_nullifier_inclusion(init_nullifier, *context);
}

pub fn compute_contract_initialization_nullifier<TContext>(context: TContext) -> Field where TContext: ContextInterface {
    let address = context.this_address();
    silo_nullifier(
        address,
        compute_unsiloed_contract_initialization_nullifier(context)
    )
}

pub fn compute_unsiloed_contract_initialization_nullifier<TContext>(context: TContext) -> Field where TContext: ContextInterface {
    context.this_address().to_field()
}

pub fn assert_initialization_matches_address_preimage<TContext>(context: TContext) where TContext: ContextInterface {
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
