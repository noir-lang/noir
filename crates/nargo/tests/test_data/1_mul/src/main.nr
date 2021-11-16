mod foo;
// This is a comment.
//
// `main` is the entry point to a binary
//
// You can have a `Binary` or a `Library`
// Release : 0.2
//
// To run a proof on the command line, type `cargo run prove {proof_name}`
//
// To verify that proof, type `cargo run verify {proof_name}`

fn main(x : u32, y : u32, z : u32) {
    x = x * y;
    x = x * x;  //144
    x = x * x;  //20736
    x = x * x;  //429 981 696
    constrain (x) == z;
}