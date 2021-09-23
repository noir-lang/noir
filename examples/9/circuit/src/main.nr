use dep::std;

fn main(message : [5]u8, root : Field) {

    // The problem with this method here is that we now need to ensure that hash_to_field
    // matches with whatever method is being used to hash the messages to leaves right?
    // We could add a "merkle_leaf_hash" method with inputs being []Field?
    //
    // Or we could make check_membership, take []u8 . This may impose a bunch of unnecessary constraints
    // Eg. If the message itself can fit into a field element, this would decompose it to u8s
    // Check with Charlie and Zac, if this is okay
    priv leaf = std::hash::hash_to_field(message);

    priv is_member = std::merkle::check_membership(root, leaf, 3);

    constrain is_member == 1;
}
