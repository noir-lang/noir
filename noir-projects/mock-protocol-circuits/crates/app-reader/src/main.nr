use dep::mock_types::{AppPublicInputs, MAX_COMMITMENT_READ_REQUESTS_PER_CALL};

// Mock app for testing that reads the commitments (generates read requests) the user commands.
// Note: A zero read is a null read.
fn main(commitments_to_read: [Field; MAX_COMMITMENT_READ_REQUESTS_PER_CALL]) -> pub AppPublicInputs {
    let mut result = AppPublicInputs::default();
    result.read_requests = commitments_to_read;
    result
}

