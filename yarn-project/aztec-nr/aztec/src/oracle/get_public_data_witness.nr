use dep::protocol_types::constants::PUBLIC_DATA_TREE_HEIGHT;
use dep::protocol_types::hash::pedersen_hash;
use crate::utils::arr_copy_slice;

global LEAF_PREIMAGE_LENGTH: Field = 4;
// TODO: move this to constants_gen.nr so that it gets computed as INDEX_LENGTH + LEAF_DATA_LENGTH + PUBLIC_DATA_TREE_HEIGHT
global PUBLIC_DATA_WITNESS: Field = 45;

// TODO(#3470) replace with /mnt/user-data/jan/aztec-packages/yarn-project/noir-protocol-circuits/src/crates/rollup-lib/src/abis/public_data_tree_leaf.nr
struct PublicDataTreeLeafPreimage {
    slot : Field,
    value: Field,
    next_index : u32,
    next_slot :Field,
}

impl PublicDataTreeLeafPreimage {
    fn serialize(self) -> [Field; LEAF_PREIMAGE_LENGTH] {
        [self.slot, self.value, self.next_index as Field, self.next_slot]
    }

    fn hash(self) -> Field {
        // Performs the same hashing as StandardIndexedTree::encodeLeaf(...)
        pedersen_hash(self.serialize(), 0)
    }
}

struct PublicDataWitness {
    index: Field,
    leaf_preimage: PublicDataTreeLeafPreimage,
    path: [Field; PUBLIC_DATA_TREE_HEIGHT],
}

#[oracle(getPublicDataTreeWitness)]
fn get_public_data_witness_oracle(_block_number: u32, _leaf_slot: Field) -> [Field; PUBLIC_DATA_WITNESS] {}

unconstrained pub fn get_public_data_witness(
    block_number: u32,
    leaf_slot: Field
) -> PublicDataWitness {
    let fields = get_public_data_witness_oracle(block_number, leaf_slot);
    PublicDataWitness {
        index: fields[0],
        leaf_preimage: PublicDataTreeLeafPreimage { slot: fields[1], value: fields[2], next_index: fields[3] as u32, next_slot: fields[4] },
        path: arr_copy_slice(fields, [0; PUBLIC_DATA_TREE_HEIGHT], 1 + LEAF_PREIMAGE_LENGTH)
    }
}
