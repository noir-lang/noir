use crate::abis::{membership_witness::NoteHashReadRequestMembershipWitness, side_effect::SideEffect};
use crate::tests::fixtures;
use crate::constants::{MAX_NOTE_HASH_READ_REQUESTS_PER_CALL};

pub fn generate_note_hash_read_requests(how_many: u64) -> (BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>, BoundedVec<NoteHashReadRequestMembershipWitness, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>) {
    generate_note_hash_read_requests_with_config(how_many, false, [0; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL])
}

pub fn generate_transient_note_hash_read_requests(how_many: u64) -> (BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>, BoundedVec<NoteHashReadRequestMembershipWitness, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>) {
    generate_note_hash_read_requests_with_config(how_many, true, [0; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL])
}

fn generate_note_hash_read_requests_with_config(
    how_many: u64,
    is_transient: bool,
    hints_to_commitment: [Field; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL]
) -> (BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>, BoundedVec<NoteHashReadRequestMembershipWitness, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>) {
    let mut read_requests = BoundedVec::new();
    let mut read_request_membership_witnesses = BoundedVec::new();

    for i in 0..how_many {
        read_requests.push(SideEffect { value: (i as Field) + 1, counter: 0 });
        let witness = NoteHashReadRequestMembershipWitness {
            leaf_index: i as Field,
            sibling_path: fixtures::note_hash_tree::SIBLING_PATHS[i],
            is_transient,
            hint_to_note_hash: hints_to_commitment[i]
        };
        read_request_membership_witnesses.push(witness);
    }

    (read_requests, read_request_membership_witnesses)
}
