use dep::std;

fn main(
    old_root : Field,
    old_leaf : Field,
    old_hash_path : [6]Field,
    updated_root : pub Field,
    leaf : Field,
    index : Field
) {
    let old_leaf_exists = std::merkle::check_membership(old_root, old_leaf, index, old_hash_path);
    constrain old_leaf_exists == 1;

    let new_leaf_exists = std::merkle::check_membership(updated_root, leaf, index, old_hash_path);
    constrain new_leaf_exists == 1;
}
    