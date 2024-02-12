use dep::protocol_types::{
    abis::nullifier_leaf_preimage::{NullifierLeafPreimage, NULLIFIER_LEAF_PREIMAGE_LENGTH},
    constants::NULLIFIER_TREE_HEIGHT, hash::pedersen_hash, utils::arr_copy_slice
};

// INDEX_LENGTH + NULLIFIER_LEAF_PREIMAGE_LENGTH + NULLIFIER_TREE_HEIGHT
global NULLIFIER_MEMBERSHIP_WITNESS: Field = 24;

struct NullifierMembershipWitness {
    index: Field,
    leaf_preimage: NullifierLeafPreimage,
    path: [Field; NULLIFIER_TREE_HEIGHT],
}

impl NullifierMembershipWitness {
    pub fn deserialize(fields: [Field; NULLIFIER_MEMBERSHIP_WITNESS]) -> Self {
        let leaf_preimage_fields = arr_copy_slice(fields, [0; NULLIFIER_LEAF_PREIMAGE_LENGTH], 1);
        Self {
            index: fields[0],
            leaf_preimage: NullifierLeafPreimage::deserialize(leaf_preimage_fields),
            path: arr_copy_slice(
                fields,
                [0; NULLIFIER_TREE_HEIGHT],
                1 + NULLIFIER_LEAF_PREIMAGE_LENGTH
            )
        }
    }
}

#[oracle(getLowNullifierMembershipWitness)]
fn get_low_nullifier_membership_witness_oracle(
    _block_number: u32,
    _nullifier: Field
) -> [Field; NULLIFIER_MEMBERSHIP_WITNESS] {}

// Nullifier here refers to the nullifier we are looking to get non-inclusion proof for (by proving that a lower
// nullifier's next_value is bigger than the nullifier)
unconstrained pub fn get_low_nullifier_membership_witness(block_number: u32, nullifier: Field) -> NullifierMembershipWitness {
    let fields = get_low_nullifier_membership_witness_oracle(block_number, nullifier);
    NullifierMembershipWitness::deserialize(fields)
}

#[oracle(getNullifierMembershipWitness)]
fn get_nullifier_membership_witness_oracle(
    _block_number: u32,
    _nullifier: Field
) -> [Field; NULLIFIER_MEMBERSHIP_WITNESS] {}

// Nullifier here refers to the nullifier we are looking to get non-inclusion proof for (by proving that a lower
// nullifier's next_value is bigger than the nullifier)
unconstrained pub fn get_nullifier_membership_witness(block_number: u32, nullifier: Field) -> NullifierMembershipWitness {
    let fields = get_nullifier_membership_witness_oracle(block_number, nullifier);
    NullifierMembershipWitness::deserialize(fields)
}
