use dep::std;

fn main(
    old_root : Field,
    leaf : Field,
    index : Field,
    hash_path : [6]Field,
    updated_root : pub Field
) {
    let result_root = std::merkle::insert(old_root, leaf, index, hash_path);

    constrain updated_root == result_root;
}
    