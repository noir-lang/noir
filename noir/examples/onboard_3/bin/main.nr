   
// Here is an example of us computing the root for a merkle tree in 
// a circuit and verifying that it is correct

// Notice here that pre-alpha dependencies must be prefixed with the word "dep".
// The line below indicates that we would like to pull in the standard library dependency. 
use dep::std;

fn main(leaf : Witness,root_pub : Public) {

    priv root = std::merkle::compute_root([leaf,leaf,leaf,leaf,leaf,leaf,leaf,leaf]);
 
    constrain root == root_pub
}