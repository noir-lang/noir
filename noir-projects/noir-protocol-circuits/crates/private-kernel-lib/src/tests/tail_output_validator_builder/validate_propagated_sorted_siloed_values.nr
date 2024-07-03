use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::tests::utils::swap_items;

/**
 * note_hashes
 */

#[test]
fn validate_propagated_sorted_siloed_values_note_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    builder.output.append_siloed_note_hashes(3);

    builder.validate();
}

#[test]
fn validate_propagated_sorted_siloed_values_note_hashes_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    swap_items(&mut builder.previous_kernel.note_hashes, 0, 2);
    for i in 0..3 {
        // Need to silo the note hashes in the right order to hash with the correct index.
        builder.output.add_siloed_note_hash(builder.previous_kernel.note_hashes.storage[i].value());
    }
    swap_items(&mut builder.output.note_hashes, 0, 2);

    builder.validate();
}

#[test(should_fail_with="mismatch sorted values")]
fn validate_propagated_sorted_siloed_values_note_hashes_mismatch_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(2);
    builder.output.append_siloed_note_hashes(2);
    // Tweak the hash in the output.
    builder.output.note_hashes.storage[0].note_hash.value += 1;

    builder.validate();
}

/**
 * nullifiers
 */

#[test]
fn validate_propagated_sorted_siloed_values_nullifiers_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_siloed_nullifiers(3);

    builder.validate();
}

#[test]
fn validate_propagated_sorted_siloed_values_nullifiers_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    swap_items(&mut builder.previous_kernel.nullifiers, 0, 3);
    builder.output.append_siloed_nullifiers(3);

    builder.validate();
}

#[test(should_fail_with="mismatch sorted values")]
fn validate_propagated_sorted_siloed_values_nullifiers_mismatch_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_siloed_nullifiers(3);
    // Tweak the hash in the output.
    builder.output.nullifiers.storage[0].nullifier.value += 1;

    builder.validate();
}

/**
 * l2_to_l1_msgs
 */

#[test]
fn validate_propagated_sorted_siloed_values_l2_to_l1_msgs_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_siloed_l2_to_l1_msgs(2);

    builder.validate();
}

#[test]
fn validate_propagated_sorted_siloed_values_l2_to_l1_msgs_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    swap_items(&mut builder.previous_kernel.l2_to_l1_msgs, 0, 1);
    builder.output.append_siloed_l2_to_l1_msgs(2);

    builder.validate();
}

#[test(should_fail_with="mismatch sorted values")]
fn validate_propagated_sorted_siloed_values_l2_to_l1_msgs_mismatch_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_siloed_l2_to_l1_msgs(2);
    // Tweak the content in the output.
    builder.output.l2_to_l1_msgs.storage[0].message.content += 1;

    builder.validate();
}
