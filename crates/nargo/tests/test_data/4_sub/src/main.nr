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
    x = x - y;
    constrain (x) == z;
}