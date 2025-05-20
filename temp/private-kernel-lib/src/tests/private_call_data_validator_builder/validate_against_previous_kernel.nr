use crate::{
    components::private_call_data_validator::PrivateCallDataValidator,
    tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder,
};
use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    tests::fixture_builder::FixtureBuilder,
};

impl PrivateCallDataValidatorBuilder {
    pub fn validate_against_previous_kernel(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
    ) {
        let private_call = self.private_call.to_private_call_data();
        PrivateCallDataValidator::new(private_call).validate_against_previous_kernel(
            previous_kernel,
        );
    }

    fn make_previous_kernel(self) -> PrivateKernelCircuitPublicInputs {
        let mut builder = FixtureBuilder::new();
        builder.historical_header = self.private_call.historical_header;
        builder.to_private_kernel_circuit_public_inputs()
    }
}

#[test]
fn validate_against_previous_kernel_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let previous_kernel = builder.make_previous_kernel();

    builder.validate_against_previous_kernel(previous_kernel);
}

#[test(should_fail_with = "mismatch historical header")]
fn validate_against_previous_kernel_mismatch_header_version_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut previous_kernel = builder.make_previous_kernel();
    previous_kernel.constants.historical_header.global_variables.version += 1;

    builder.validate_against_previous_kernel(previous_kernel);
}

#[test(should_fail_with = "mismatch tx context")]
fn validate_against_previous_kernel_mismatch_chain_id_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut previous_kernel = builder.make_previous_kernel();
    previous_kernel.constants.tx_context.chain_id += 1;

    builder.validate_against_previous_kernel(previous_kernel);
}
