   
// Here is an example of checking whether a leaf is in a tree at a specified index

use dep::std;

fn main(leaf : Field, index : Field, hashpath : [6]Field, root : Field) {

    const one = 1;

    priv is_member = std::merkle::check_membership(root, leaf, index, hashpath);

    constrain is_member == one;
}
