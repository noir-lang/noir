use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};
use crate::{context::PrivateContext, state_vars::private_mutable::PrivateMutable};
use crate::test::{mocks::mock_note::MockNote, helpers::context_builder::ContextBuilder};
use dep::std::{unsafe::zeroed, test::OracleMock};

global contract_address = AztecAddress::from_field(13);
global storage_slot = 17;

fn setup() -> PrivateMutable<MockNote, &mut PrivateContext> {
    let mut context = ContextBuilder::new().contract_address(contract_address).private();
    let state_var = PrivateMutable::new(&mut context, storage_slot);

    // This oracle is called for its side effects alone - it's always expected to return 0.
    OracleMock::mock("notifyCreatedNote").returns(0);

    state_var
}

#[test]
fn test_initialize_or_replace_without_nullifier() {
    let state_var = setup();

    let ovpk_m: GrumpkinPoint = zeroed();
    let ivpk_m: GrumpkinPoint = zeroed();

    let value = 42;
    let mut note = MockNote::new(value).contract_address(contract_address).storage_slot(storage_slot).build();

    OracleMock::mock("checkNullifierExists").returns(0);
    state_var.initialize_or_replace(&mut note, ovpk_m, ivpk_m);

    // Since we reported there was no nullifier, we should initialize and see the following side-effects:
    // - a new note being created
    // - no notes being read
    // - the initialization nullifier being emitted
    assert_eq(state_var.context.new_note_hashes.len(), 1);
    assert_eq(state_var.context.note_hash_read_requests.len(), 0);
    assert_eq(state_var.context.new_nullifiers.len(), 1);

    // Note that if the oracle was wrong and the initialization nullifier did exist, this attempt to write it again
    // would cause the sequencer to revert this transaction - we are therefore safe from bad oracles.
    let nullifier = state_var.context.new_nullifiers.get(0);
    assert_eq(nullifier.value, state_var.compute_initialization_nullifier());
    assert_eq(nullifier.note_hash, 0);
}

#[test]
fn test_initialize_or_replace_with_nullifier() {
    // Here we'd want to test a scenario like the one above with the oracle indicating that the initialization
    // nullifier does exist. Unfortunately that requires us to produce a valid oracle return value for getNotes,
    // which is fairly involved as it deals with serialization of notes, and is relatively complicated to replicate
    // in Noir.
}
