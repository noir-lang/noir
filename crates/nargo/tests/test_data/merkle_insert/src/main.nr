use dep::std;

fn main(
    old_root : Field,
    old_hash_path : [6]Field,
    updated_root : pub Field,
    old_leaf : Field,
    leaf : Field,
    index : Field
) {
    let result_empty_leaf = std::merkle::check_membership(old_root, old_leaf, index, old_hash_path);
    constrain result_empty_leaf == 1;

    let result_leaf_in_position = std::merkle::check_membership(updated_root, leaf, index, old_hash_path);
    constrain result_leaf_in_position == 1;
}
    