use dep::protocol_types::constants::NOTE_HASH_TREE_HEIGHT;
use crate::utils::arr_copy_slice;

// Note: We have M here because we need to somehow set it when calling get_membership_witness function and one way to
// do it is to set M here and then set type of the return param, e.g.:
//
// `let witness: MembershipWitness<NOTE_HASH_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT + 1> = get_membership_witness(...);`
//
// Another way to do it would be to add "type_hint: [Field; T]" as argument to `get_membership_witness` but that's
// a bit too boilerplatey for my taste.
struct MembershipWitness<N, M> {
    index: Field,
    path: [Field; N],
}

#[oracle(getMembershipWitness)]
fn get_membership_witness_oracle<M>(_block_number: u32, _tree_id: Field, _leaf_value: Field) -> [Field; M] {}

unconstrained pub fn get_membership_witness<N, M>(block_number: u32, tree_id: Field, leaf_value: Field) -> MembershipWitness<N, M> {
    let fields: [Field; M] = get_membership_witness_oracle(block_number, tree_id, leaf_value);
    MembershipWitness { index: fields[0], path: arr_copy_slice(fields, [0; N], 1) }
}
