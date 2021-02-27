
// This should not compile as the keyword 
// is `constrain` and not `constrai`

fn main(x : Witness, y : Witness) {
    constrai x != y;
}