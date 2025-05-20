use crate::tests::reset_output_validator_builder::ResetOutputValidatorBuilder;
use dep::types::tests::utils::swap_items;

impl ResetOutputValidatorBuilder {
    pub fn new_with_note_hash_siloing() -> Self {
        let mut builder = ResetOutputValidatorBuilder::new();
        builder.note_hash_siloing_amount = 6;
        builder
    }
}

#[test]
fn validate_sorted_siloed_note_hashes_succeeds() {
    let mut builder = ResetOutputValidatorBuilder::new_with_note_hash_siloing();

    builder.previous_kernel.append_note_hashes(4);
    builder.output.append_siloed_note_hashes(4);

    builder.validate();
}

#[test]
fn validate_sorted_siloed_note_hashes_unordered_succeeds() {
    let mut builder = ResetOutputValidatorBuilder::new_with_note_hash_siloing();

    builder.previous_kernel.append_note_hashes(4);
    swap_items(&mut builder.previous_kernel.note_hashes, 0, 2);

    builder.output.append_siloed_note_hashes(4);

    builder.validate();
}

#[test(should_fail_with = "incorrect siloed note hashes")]
fn validate_sorted_siloed_note_hashes_mismatch_sorted_hash_fails() {
    let mut builder = ResetOutputValidatorBuilder::new_with_note_hash_siloing();

    builder.previous_kernel.append_note_hashes(2);

    builder.output.append_siloed_note_hashes(2);
    // Swap the hashes in the output.
    let mut note_hash_0 = builder.output.note_hashes.get(0);
    let mut note_hash_1 = builder.output.note_hashes.get(1);
    let tmp = note_hash_0.note_hash.value;
    note_hash_0.note_hash.value = note_hash_1.note_hash.value;
    note_hash_1.note_hash.value = tmp;
    builder.output.note_hashes.set(0, note_hash_0);
    builder.output.note_hashes.set(1, note_hash_1);

    builder.validate();
}

#[test(should_fail_with = "unexpected non-empty item")]
fn validate_sorted_siloed_note_hashes_extra_item_fails() {
    let mut builder = ResetOutputValidatorBuilder::new_with_note_hash_siloing();

    builder.previous_kernel.append_note_hashes(2);

    // Append an extra item to the output.
    builder.output.append_siloed_note_hashes(3);
    let mut hints = builder.get_hints();
    // Tweak the hint to point to an empty item.
    hints.sorted_note_hash_indexes[2] = 3;

    builder.validate_with_hints(hints);
}

#[test(should_fail_with = "note hashes have been siloed in a previous reset")]
fn validate_sorted_siloed_note_hashes_repeat_siloing_fails() {
    let mut builder = ResetOutputValidatorBuilder::new_with_note_hash_siloing();

    // Add a siloed note hash to the previous kernel.
    builder.previous_kernel.append_siloed_note_hashes(1);
    builder.previous_kernel.append_note_hashes(2);

    builder.output.append_siloed_note_hashes(3);

    builder.validate();
}
