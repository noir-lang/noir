// This file contains items which are mocked because 
// they need the proof system.
use crate::traits::Empty;

struct AggregationObject{}

struct Proof{}

struct VerificationKey{}

impl Empty for AggregationObject {
    fn empty() -> Self {
        AggregationObject {}
    }
}

impl Empty for Proof {
    fn empty() -> Self {
        Proof {}
    }
}

impl Empty for VerificationKey {
    fn empty() -> Self {
        VerificationKey {}
    }
}

// Perform recursive verification of the previous kernel proof and private call
// proof.
pub fn verify_previous_kernel_state(
    _aggregation_object: AggregationObject,
    _proof: Proof
) -> (bool, AggregationObject) {
    (true, AggregationObject {})
}

// Called by private kernel init circuit which has no previous kernel proof.
pub fn verify_private_function_proof(_proof: Proof) -> bool {
    true
}
