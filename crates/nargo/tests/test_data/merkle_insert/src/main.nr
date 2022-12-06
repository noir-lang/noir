use dep::std;

fn main(
    old_root: Field,
    old_leaf: Field,
    old_hash_path: [Field; 3],
    new_root: pub Field,
    leaf: Field,
    index: Field,
    mimc_input: [Field; 4],
) {
    // The foreign check_membership function is currently broken
    // let old_leaf_exists = std::merkle::check_membership(old_root, old_leaf, index, old_hash_path);
    // constrain old_leaf_exists == 1;
    constrain old_root == std::merkle::compute_root_from_leaf(old_leaf, index, old_hash_path);
    // let new_leaf_exists = std::merkle::check_membership(new_root, leaf, index, old_hash_path);
    // constrain new_leaf_exists == 1;
    constrain new_root == std::merkle::compute_root_from_leaf(leaf, index, old_hash_path);

    let h = std::hash::mimc_bn254(mimc_input);
    constrain h == 18226366069841799622585958305961373004333097209608110160936134895615261821931;
}
    
