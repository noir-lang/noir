use dep::std;

// This test checks that we perform dead-instruction-elimination on intrinsic functions.

fn main(x: Field) {
    let bytes = x.to_be_bytes(32);
  
    let hash = std::hash::pedersen([x]);
    let _p1 = std::scalar_mul::fixed_base_embedded_curve(x);

}
