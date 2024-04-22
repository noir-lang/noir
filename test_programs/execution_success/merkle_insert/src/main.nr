use dep::std;
use dep::std::hash::mimc;

fn main(
    old_root: field,
    old_leaf: field,
    old_hash_path: [field; 3],
    new_root: pub field,
    leaf: field,
    index: field,
    mimc_input: [field; 4]
) {
    assert(old_root == std::merkle::compute_merkle_root(old_leaf, index, old_hash_path));

    let calculated_root = std::merkle::compute_merkle_root(leaf, index, old_hash_path);
    assert(new_root == calculated_root);

    let h = mimc::mimc_bn254(mimc_input);
    // Regression test for PR #891
    std::println(h);
    assert(h == 18226366069841799622585958305961373004333097209608110160936134895615261821931);
}
