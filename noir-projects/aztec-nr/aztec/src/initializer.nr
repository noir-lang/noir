use dep::protocol_types::hash::silo_nullifier;
use crate::context::PrivateContext;

pub fn mark_as_initialized(context: &mut PrivateContext) {
  let init_nullifier = compute_unsiloed_contract_initialization_nullifier(context);
  context.push_new_nullifier(init_nullifier, 0);

  // We push a commitment as well and use this value to check initialization,
  // since we cannot yet read a nullifier from the same tx in which it was emitted.
  // Eventually, when that's supported, we should delete this note_hash and
  // have all checks rely on reading the nullifier directly.
  // TODO(@spalladino) Remove when possible.
  context.push_new_note_hash(init_nullifier);
}

// TODO(@spalladino): Add a variant using PublicContext once we can read nullifiers or note hashes from public-land.
pub fn assert_is_initialized(context: &mut PrivateContext) {
  let init_nullifier = compute_contract_initialization_nullifier(context);
  context.push_read_request(init_nullifier);
}

pub fn compute_contract_initialization_nullifier(context: &mut PrivateContext) -> Field {
  let address = context.this_address();
  silo_nullifier(address, compute_unsiloed_contract_initialization_nullifier(context))
}

pub fn compute_unsiloed_contract_initialization_nullifier(context: &mut PrivateContext) -> Field {
  context.this_address().to_field()
}
