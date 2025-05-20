use crate::tests::reset_output_validator_builder::ResetOutputValidatorBuilder;
use dep::types::tests::utils::swap_items;

impl ResetOutputValidatorBuilder {
    pub fn new_with_nullifier_siloing() -> Self {
        let mut builder = ResetOutputValidatorBuilder::new();
        builder.nullifier_siloing_amount = 6;
        builder
    }
}

#[test]
fn validate_sorted_siloed_nullifiers_succeeds() {
    let mut builder = ResetOutputValidatorBuilder::new_with_nullifier_siloing();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_siloed_nullifiers(3);

    builder.validate();
}

#[test]
fn validate_sorted_siloed_nullifiers_unordered_succeeds() {
    let mut builder = ResetOutputValidatorBuilder::new_with_nullifier_siloing();

    builder.previous_kernel.append_nullifiers(3);
    swap_items(&mut builder.previous_kernel.nullifiers, 1, 2);

    builder.output.append_siloed_nullifiers(3);

    builder.validate();
}

#[test]
fn validate_sorted_siloed_nullifiers_repeatedly_succeeds() {
    let mut builder = ResetOutputValidatorBuilder::new_with_nullifier_siloing();

    // 2 nullifiers are already siloed.
    builder.previous_kernel.append_siloed_nullifiers(2);
    builder.previous_kernel.append_nullifiers(1);

    builder.output.append_siloed_nullifiers(3);

    builder.validate();
}

#[test(should_fail_with = "incorrect transformed value")]
fn validate_sorted_siloed_nullifiers_mismatch_sorted_hash_fails() {
    let mut builder = ResetOutputValidatorBuilder::new_with_nullifier_siloing();

    builder.previous_kernel.append_nullifiers(3);

    builder.output.append_siloed_nullifiers(3);
    // Swap the hashes in the output.
    let mut nullifier_0 = builder.output.nullifiers.get(0);
    let mut nullifier_1 = builder.output.nullifiers.get(1);
    let tmp = nullifier_0.nullifier.value;
    nullifier_0.nullifier.value = nullifier_1.nullifier.value;
    nullifier_1.nullifier.value = tmp;
    builder.output.nullifiers.set(0, nullifier_0);
    builder.output.nullifiers.set(1, nullifier_1);

    builder.validate();
}

#[test(should_fail_with = "unexpected non-empty item")]
fn validate_sorted_siloed_nullifiers_extra_item_fails() {
    let mut builder = ResetOutputValidatorBuilder::new_with_nullifier_siloing();

    builder.previous_kernel.append_nullifiers(2);

    // Append an extra item to the output.
    builder.output.append_siloed_nullifiers(3);
    let mut hints = builder.get_hints();
    // Tweak the hint to point to an empty item.
    hints.sorted_nullifier_indexes[3] = 4;

    builder.validate_with_hints(hints);
}
