use crate::tests::previous_kernel_validator_builder::PreviousKernelValidatorBuilder;

impl PreviousKernelValidatorBuilder {
    pub fn append_same_inner_note_hashes(&mut self) -> [Field; 4] {
        let mut siloed_note_hashes = [0; 4];
        let inner_note_hash = 123123;
        for i in 0..siloed_note_hashes.len() {
            self.previous_kernel.add_siloed_note_hash(inner_note_hash);
            siloed_note_hashes[i] =
                self.previous_kernel.note_hashes.get_unchecked(i).note_hash.value;
        }
        siloed_note_hashes
    }
}

#[test]
fn validate_no_transient_data_no_extra_nullifiers_succeeds() {
    let builder = PreviousKernelValidatorBuilder::new_tail();

    builder.validate_for_private_tail();
}

#[test]
fn validate_no_transient_data_no_transient_nullifiers_succeeds() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail();

    builder.previous_kernel.append_siloed_nullifiers(3);

    builder.validate_for_private_tail();
}

#[test]
fn validate_no_transient_data_nullifiers_for_note_hashes_succeeds() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail();

    let siloed_note_hashes = builder.append_same_inner_note_hashes();
    builder.previous_kernel.add_siloed_nullifier(1);
    builder.previous_kernel.add_siloed_nullifier_for_note_hash(2, siloed_note_hashes[2]);
    builder.previous_kernel.add_siloed_nullifier_for_note_hash(3, siloed_note_hashes[0]);
    builder.previous_kernel.add_siloed_nullifier(4);

    builder.validate_for_private_tail();
}

#[test(should_fail_with = "Cannot link a note hash emitted after a nullifier")]
fn validate_no_transient_data_nullifiers_for_note_hashes_emitted_after_fails() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail();

    builder.previous_kernel.append_siloed_nullifiers(2);
    // Emit the note hashes after the nullifiers
    let siloed_note_hashes = builder.append_same_inner_note_hashes();
    // Link a note hash to the nullifier at index 2.
    let mut nullifier = builder.previous_kernel.nullifiers.get(2);
    nullifier.nullifier.note_hash = siloed_note_hashes[0];
    builder.previous_kernel.nullifiers.set(2, nullifier);

    builder.validate_for_private_tail();
}

#[test(should_fail_with = "Hinted siloed note hash does not match nullified note hash")]
fn validate_no_transient_data_nullifiers_for_note_hashes_not_found_fails() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail();

    let siloed_note_hashes = builder.append_same_inner_note_hashes();
    builder.previous_kernel.add_siloed_nullifier(1);
    builder.previous_kernel.add_siloed_nullifier_for_note_hash(2, siloed_note_hashes[2]);
    // Assign a random note hash for nullifier 3.
    builder.previous_kernel.add_siloed_nullifier_for_note_hash(3, 123);
    builder.previous_kernel.add_siloed_nullifier(4);

    builder.validate_for_private_tail();
}
