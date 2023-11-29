use crate::constants_gen::NULLIFIER_TREE_HEIGHT;
use crate::utils::arr_copy_slice;
use crate::hash::pedersen_hash;

global LEAF_DATA_LENGTH: Field = 3;
// TODO: move this to constants.hpp so that it gets computed as INDEX_LENGTH + LEAF_DATA_LENGTH + NULLIFIER_TREE_HEIGHT
global NULLIFIER_MEMBERSHIP_WITNESS: Field = 24;

// Noir version of LeafData interface from indexed merkle tree.
// TODO(#3470) replace with /mnt/user-data/jan/aztec-packages/yarn-project/noir-protocol-circuits/src/crates/rollup-lib/src/abis/nullifier_leaf_preimage.nr
struct LeafData {
    value: Field,
    next_index: Field,
    next_value: Field,
}

impl LeafData {
    fn serialize(self) -> [Field; LEAF_DATA_LENGTH] {
        [self.value, self.next_index, self.next_value]
    }

    fn hash(self) -> Field {
        // Performs the same hashing as StandardIndexedTree::encodeLeaf(...)
        pedersen_hash(self.serialize(), 0)
    }
}

struct NullifierMembershipWitness {
    index: Field,
    leaf_data: LeafData,
    path: [Field; NULLIFIER_TREE_HEIGHT],
}

#[oracle(getLowNullifierMembershipWitness)]
fn get_low_nullifier_membership_witness_oracle(_block_number: Field, _nullifier: Field) -> [Field; NULLIFIER_MEMBERSHIP_WITNESS] {}

// Nullifier here refers to the nullifier we are looking to get non-inclusion proof for (by proving that a lower
// nullifier's next_value is bigger than the nullifier)
unconstrained pub fn get_low_nullifier_membership_witness(block_number: Field, nullifier: Field) -> NullifierMembershipWitness {
    let fields = get_low_nullifier_membership_witness_oracle(block_number, nullifier);
    NullifierMembershipWitness {
        index: fields[0],
        leaf_data: LeafData { value: fields[1], next_index: fields[2], next_value: fields[3] },
        path: arr_copy_slice(fields, [0; NULLIFIER_TREE_HEIGHT], 1 + LEAF_DATA_LENGTH)
    }
}

#[oracle(getNullifierMembershipWitness)]
fn get_nullifier_membership_witness_oracle(_block_number: Field, _nullifier: Field) -> [Field; NULLIFIER_MEMBERSHIP_WITNESS] {}

// Nullifier here refers to the nullifier we are looking to get non-inclusion proof for (by proving that a lower
// nullifier's next_value is bigger than the nullifier)
unconstrained pub fn get_nullifier_membership_witness(block_number: Field, nullifier: Field) -> NullifierMembershipWitness {
    let fields = get_nullifier_membership_witness_oracle(block_number, nullifier);
    NullifierMembershipWitness {
        index: fields[0],
        leaf_data: LeafData { value: fields[1], next_index: fields[2], next_value: fields[3] },
        path: arr_copy_slice(fields, [0; NULLIFIER_TREE_HEIGHT], 1 + LEAF_DATA_LENGTH)
    }
}