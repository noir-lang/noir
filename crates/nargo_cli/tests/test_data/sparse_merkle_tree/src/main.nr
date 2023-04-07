use dep::std;
use dep::std::hash;
use dep::std::merkle;
use dep::std::merkle::smt::SparseMerkleTree;

fn main(
    leaf_index: Field,
    new_leaf1: Field,
    new_leaf2: Field,
) -> pub [Field; 3] {
    let default_value = 0xdead;
    let hash_low = hash::pedersen([default_value, default_value])[0];
    let hash_mid = hash::pedersen([hash_low, hash_low])[0];
    let hash_high = hash::pedersen([hash_mid, hash_mid])[0];
    let empty_root = hash::pedersen([hash_high, hash_high])[0];
    
    // Remains the same since we're only testing mutations at a single index
    let hash_path = [default_value, hash_low, hash_mid, hash_high];

    let root_after_insert_leaf1 = merkle::compute_root_from_leaf(
        new_leaf1,
        leaf_index,
        hash_path,
    );
    let root_after_replace_leaf1_with_leaf2 = merkle::compute_root_from_leaf(
        new_leaf2,
        leaf_index,
        hash_path,
    );

    let smt = SparseMerkleTree { default_value };

    smt.insert(
        empty_root,
        root_after_insert_leaf1,
        new_leaf1,
        leaf_index,
        hash_path,
    );

    SparseMerkleTree::update(
        root_after_insert_leaf1, // old_root
        new_leaf1, // old leaf
        root_after_replace_leaf1_with_leaf2, // new_root
        new_leaf2,
        leaf_index,
        hash_path,
    );

    smt.delete(
        root_after_replace_leaf1_with_leaf2, // old_root
        new_leaf2, // old leaf
        empty_root, // new root
        leaf_index,
        hash_path
    );

    [
        empty_root,
        root_after_insert_leaf1,
        root_after_replace_leaf1_with_leaf2
    ]
}
    
