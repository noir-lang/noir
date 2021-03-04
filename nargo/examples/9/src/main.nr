   
// Here is an example of checking whether a leaf is in a tree at a specified index

use dep::std;

fn main(message : [64]u8, index : Field, hashpath : [6]Field, root : Field) {

    priv leaf = std::hash::hash_to_field(message);

    priv is_member = std::merkle::check_membership(root, leaf, index, hashpath);

    constrain is_member == 1;
}
