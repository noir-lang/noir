// This file contains items which are mocked because 
// they need the proof system.

struct AggregationObject{}

struct Proof{}

struct VerificationKey{}

// Perform recursive verification of the previous kernel proof and private call
// proof.
pub fn verify_previous_kernel_state(
    _aggregation_object: AggregationObject,
    _proof: Proof
) -> (bool, AggregationObject) {
    (true, AggregationObject {})
}
