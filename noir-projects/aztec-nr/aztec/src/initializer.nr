use dep::protocol_types::hash::silo_nullifier;
use crate::context::{PrivateContext, PublicContext, ContextInterface};
use crate::history::nullifier_inclusion::prove_nullifier_inclusion;

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
