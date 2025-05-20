use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;

impl TailOutputValidatorBuilder {
    pub fn new_with_data() -> Self {
        let mut builder = TailOutputValidatorBuilder::new();

        builder.previous_kernel.append_siloed_note_hashes(3);
        builder.output.append_siloed_note_hashes(3);

        builder.previous_kernel.append_private_logs(3);
        builder.output.append_private_logs(3);

        builder
    }
}

#[test]
fn validate_gas_used_succeeds() {
    let builder = TailOutputValidatorBuilder::new_with_data();
    let output = builder.export_output();
    builder.validate_with_output(output);
}

#[test(should_fail_with = "incorrect metered gas used")]
fn validate_gas_used_wrong_da_gas_fails() {
    let builder = TailOutputValidatorBuilder::new_with_data();
    let mut output = builder.export_output();

    // Tweak the da gas in the output to be a wrong value.
    output.gas_used.da_gas += 1;

    builder.validate_with_output(output);
}

#[test(should_fail_with = "incorrect metered gas used")]
fn validate_gas_used_wrong_l2_gas_fails() {
    let builder = TailOutputValidatorBuilder::new_with_data();
    let mut output = builder.export_output();

    // Tweak the l2 gas in the output to be a wrong value.
    output.gas_used.l2_gas += 1;

    builder.validate_with_output(output);
}

#[test(should_fail_with = "The gas used exceeds the gas limits")]
fn validate_gas_used_exceed_da_limit_fails() {
    let mut builder = TailOutputValidatorBuilder::new_with_data();
    let mut output = builder.export_output();

    let gas_used = output.gas_used.da_gas;
    // Tweak the da gas limit to be less than the gas used.
    output.constants.tx_context.gas_settings.gas_limits.da_gas = gas_used - 1;
    // Constants must match.
    builder.previous_kernel.tx_context.gas_settings.gas_limits.da_gas = gas_used - 1;

    builder.validate_with_output(output);
}

#[test(should_fail_with = "The gas used exceeds the gas limits")]
fn validate_gas_used_exceed_l2_limit_fails() {
    let mut builder = TailOutputValidatorBuilder::new_with_data();
    let mut output = builder.export_output();

    let gas_used = output.gas_used.l2_gas;
    // Tweak the l2 gas limit to be less than the gas used.
    output.constants.tx_context.gas_settings.gas_limits.l2_gas = gas_used - 1;
    // Constants must match.
    builder.previous_kernel.tx_context.gas_settings.gas_limits.l2_gas = gas_used - 1;

    builder.validate_with_output(output);
}
