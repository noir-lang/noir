   
// Here is an example of checking whether a leaf is in a tree at a specified index

use dep::std;

fn main(
    leaf : Witness,
    index : Witness,
    hashpath_0 : Witness,
    hashpath_1 : Witness,
    hashpath_2 : Witness,
    hashpath_3 : Witness,
    hashpath_4 : Witness,
    hashpath_5 : Witness,
    root : Witness
) {

    const one = 1;

    priv is_member = std::merkle::check_membership(root, leaf, index, [hashpath_0, hashpath_1, hashpath_2, hashpath_3, hashpath_4, hashpath_5]);

    constrain is_member == one
}
