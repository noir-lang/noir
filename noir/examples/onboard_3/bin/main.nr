   
// Here is an example of us computing the root for a merkle tree in 
// a circuit and verifying that it is correct

// To create a smart contract for this Noir program, type `cargo run contract`

// Notice here that pre-alpha dependencies must be prefixed with the word "dep".
// The line below indicates that we would like to pull in the standard library dependency. 
use dep::std;

fn main(leaf : Witness,root_pub : Public) {

    // Notice that `root` is an array of one element.
    // This is because `compute_root` hooks directly into the ABI of ACIR 
    // Which only handles arrays.
    // 
    // An alternative is to wrap the low level functions in Normal functions
    // which will then hide this fact.
    let root = std::merkle::compute_root([leaf,leaf,leaf,leaf,leaf,leaf,leaf,leaf]);
     
    constrain root[0] == root_pub
}