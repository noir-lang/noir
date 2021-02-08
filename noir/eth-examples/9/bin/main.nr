   
// Here is an example of checking whether a leaf is in a tree at a specified index

use dep::std;

fn main(leaf : Witness, index : Witness, hashpath : [6]Witness, root : Witness) {

    const one = 1;

    priv is_member = std::merkle::check_membership(root, leaf, index, hashpath);

    constrain is_member == one
}
