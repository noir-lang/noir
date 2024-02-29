use dep::protocol_types::hash::silo_nullifier;
use crate::context::PrivateContext;
use crate::history::nullifier_inclusion::prove_nullifier_inclusion;

pub fn mark_as_initialized(context: &mut PrivateContext) {
  let init_nullifier = compute_unsiloed_contract_initialization_nullifier(*context);
  context.push_new_nullifier(init_nullifier, 0);
}

// TODO(@spalladino): Add a variant using PublicContext once we can read nullifiers or note hashes from public-land.
pub fn assert_is_initialized(context: &mut PrivateContext) {
  let init_nullifier = compute_contract_initialization_nullifier(*context);
  prove_nullifier_inclusion(init_nullifier, *context);
}

pub fn compute_contract_initialization_nullifier(context: PrivateContext) -> Field {
  let address = context.this_address();
  silo_nullifier(address, compute_unsiloed_contract_initialization_nullifier(context))
}

pub fn compute_unsiloed_contract_initialization_nullifier(context: PrivateContext) -> Field {
  context.this_address().to_field()
}
