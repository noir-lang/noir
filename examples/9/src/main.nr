   
// Here is an example of checking whether a leaf is in a tree at a specified index

use dep::std;

fn main(message : [64]u8, root : Field) {

    // The problem with this method here is that we now need to ensure that hash_to_field
    // matches with whatever method is being used to hash the messages to leaves right?
    // We could add a "merkle_leaf_hash" method
    priv leaf = std::hash::hash_to_field(message);

    priv is_member = std::merkle::check_membership(root, leaf, 3);

    constrain is_member == 1;
}
