// `inner` is written as a constrained function, but it is only reached from the unconstrained
// `hint`, so it is compiled as unconstrained and its proof verification cannot be lowered.
fn inner(vk: [Field; 1], proof: [Field; 1], pi: [Field; 1], kh: Field) {
    std::verify_proof_with_type(vk, proof, pi, kh, 0);
}

unconstrained fn hint(vk: [Field; 1], proof: [Field; 1], pi: [Field; 1], kh: Field) {
    inner(vk, proof, pi, kh);
}

fn main(vk: [Field; 1], proof: [Field; 1], pi: [Field; 1], kh: Field) {
    // Safety: `hint` returns nothing; the program only exercises the compile-time rejection.
    unsafe { hint(vk, proof, pi, kh) };
}
