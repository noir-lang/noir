use dep::std;

// This test checks that we perform dead-instruction-elimination on intrinsic functions.

fn main(x: Field) {
    let bytes = x.to_be_bytes(32);
  
    let hash = std::hash::pedersen([x]);
}
