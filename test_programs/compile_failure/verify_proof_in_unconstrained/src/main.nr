// Test that calling verify_proof_with_type in an unconstrained context produces an error.
// Proof verification is a constrained operation and cannot be meaningfully executed
// in an unconstrained context.

unconstrained fn main() {
    std::verify_proof_with_type([], [], [], 0, 0);
}
