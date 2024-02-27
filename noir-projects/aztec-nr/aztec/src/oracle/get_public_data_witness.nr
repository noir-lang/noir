use dep::protocol_types::{
    constants::PUBLIC_DATA_TREE_HEIGHT, hash::pedersen_hash,
    public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage, traits::{Hash, Serialize},
    utils::arr_copy_slice
};

global LEAF_PREIMAGE_LENGTH: u64 = 4;
global PUBLIC_DATA_WITNESS: Field = 45;

struct PublicDataWitness {
    index: Field,
    leaf_preimage: PublicDataTreeLeafPreimage,
    path: [Field; PUBLIC_DATA_TREE_HEIGHT],
}

#[oracle(getPublicDataTreeWitness)]
fn get_public_data_witness_oracle(
    _block_number: u32,
    _leaf_slot: Field
) -> [Field; PUBLIC_DATA_WITNESS] {}

unconstrained pub fn get_public_data_witness(block_number: u32, leaf_slot: Field) -> PublicDataWitness {
    let fields = get_public_data_witness_oracle(block_number, leaf_slot);
    PublicDataWitness {
        index: fields[0],
        leaf_preimage: PublicDataTreeLeafPreimage { slot: fields[1], value: fields[2], next_index: fields[3] as u64, next_slot: fields[4] },
        path: arr_copy_slice(fields, [0; PUBLIC_DATA_TREE_HEIGHT], 1 + LEAF_PREIMAGE_LENGTH)
    }
}
