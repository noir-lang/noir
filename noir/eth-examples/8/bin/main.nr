   
// Here is an example of us computing the root for a merkle tree in 
// a circuit and verifying that it is correct
//
// Note: There is a difference between the `leaf`, which is the output of the hash
// and the `message` which is the input to the hash.
// 
// Ideally, in the future we want to accept the message and hash it. (See toml file)
 
use dep::std;

fn main(leaf : Witness,root_pub : Public) {

    priv root = std::merkle::compute_root([leaf,leaf,leaf,leaf,leaf,leaf,leaf,leaf]);
 
    constrain root == root_pub
}