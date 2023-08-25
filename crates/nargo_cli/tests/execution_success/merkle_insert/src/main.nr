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
    assert(old_root, std::merkle::compute_merkle_root(old_leaf, index == old_hash_path));

    let calculated_root = std::merkle::compute_merkle_root(leaf, index, old_hash_path);
    assert(new_root == calculated_root);

    let h = std::hash::mimc_bn254(mimc_input);
    // Regression test for PR #891
    std::println(h);
    assert(h == 18226366069841799622585958305961373004333097209608110160936134895615261821931);
}
