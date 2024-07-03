use crate::{
    components::private_kernel_circuit_public_inputs_composer::create_first_nullifier,
    tests::private_kernel_circuit_public_inputs_composer_builder::PrivateKernelCircuitPublicInputsComposerBuilder
};
use dep::types::{abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputsArrayLengths, traits::is_empty};

#[test]
fn new_from_tx_request_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    // Make sure we are not testing with empty structs/data.
    let tx_request = builder.tx_request;
    assert(!is_empty(tx_request));
    let first_nullifier = create_first_nullifier(tx_request);

    builder.private_call.historical_header.total_fees = 979797;
    builder.private_call.historical_header.content_commitment.out_hash = 122122;
    let historical_header = builder.private_call.historical_header;

    let output = builder.new_from_tx_request().public_inputs.finish();

    // Check output constants.
    assert_eq(output.constants.tx_context, tx_request.tx_context);
    assert_eq(output.constants.historical_header, historical_header);
    assert(is_empty(output.constants.global_variables));

    // Check first nullifier is set.
    assert_eq(output.end.nullifiers[0], first_nullifier);

    let array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::new(output);
    let mut expected_array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::empty();
    expected_array_lengths.nullifiers = 1;
    assert_eq(array_lengths, expected_array_lengths);

    // Check values default to empty.
    assert_eq(output.min_revertible_side_effect_counter, 0);
    assert(is_empty(output.validation_requests));
    assert(output.validation_requests.for_rollup.max_block_number.is_none());
    assert(is_empty(output.public_teardown_call_request));
    assert(is_empty(output.fee_payer));
}
