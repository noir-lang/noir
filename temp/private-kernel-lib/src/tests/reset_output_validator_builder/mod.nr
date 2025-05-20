mod validate_sorted_siloed_note_hashes;
mod validate_sorted_siloed_nullifiers;

use crate::components::{
    reset_output_composer::reset_output_hints::{generate_reset_output_hints, ResetOutputHints},
    reset_output_validator::ResetOutputValidator,
};
use dep::reset_kernel_lib::{
    KeyValidationHint,
    PrivateValidationRequestProcessor,
    tests::{NoteHashReadRequestHintsBuilder, NullifierReadRequestHintsBuilder},
    TransientDataIndexHint,
};
use dep::types::{
    constants::{MAX_KEY_VALIDATION_REQUESTS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX},
    tests::fixture_builder::FixtureBuilder,
};

global NOTE_HASH_PENDING_AMOUNT: u32 = 6;
global NOTE_HASH_SETTLED_AMOUNT: u32 = 3;
global NULLIFIER_PENDING_AMOUNT: u32 = 5;
global NULLIFIER_SETTLED_AMOUNT: u32 = 2;
global NULLIFIER_KEYS: u32 = 2;
global TRANSIENT_DATA_AMOUNT: u32 = 5;

pub struct ResetOutputValidatorBuilder {
    output: FixtureBuilder,
    previous_kernel: FixtureBuilder,
    note_hash_read_request_hints_builder: NoteHashReadRequestHintsBuilder<NOTE_HASH_PENDING_AMOUNT, NOTE_HASH_SETTLED_AMOUNT>,
    nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder<NULLIFIER_PENDING_AMOUNT, NULLIFIER_SETTLED_AMOUNT>,
    key_validation_hints: [KeyValidationHint; NULLIFIER_KEYS],
    transient_data_index_hints: [TransientDataIndexHint; TRANSIENT_DATA_AMOUNT],
    note_hash_siloing_amount: u32,
    nullifier_siloing_amount: u32,
    private_log_siloing_amount: u32,
}

impl ResetOutputValidatorBuilder {
    pub fn new() -> Self {
        let mut output = FixtureBuilder::new();
        let mut previous_kernel = FixtureBuilder::new();
        output.set_protocol_nullifier();
        previous_kernel.set_protocol_nullifier();
        output.validation_requests_split_counter = Option::some(0);
        previous_kernel.validation_requests_split_counter = Option::some(0);

        let note_hash_read_request_hints_builder = NoteHashReadRequestHintsBuilder::new();
        let nullifier_read_request_hints_builder = NullifierReadRequestHintsBuilder::new();
        let key_validation_hints =
            [KeyValidationHint::nada(MAX_KEY_VALIDATION_REQUESTS_PER_TX); NULLIFIER_KEYS];
        let transient_data_index_hints = [
            TransientDataIndexHint::nada(MAX_NULLIFIERS_PER_TX, MAX_NOTE_HASHES_PER_TX);
                 TRANSIENT_DATA_AMOUNT
            ];

            ResetOutputValidatorBuilder {
                output,
                previous_kernel,
                note_hash_read_request_hints_builder,
                nullifier_read_request_hints_builder,
                key_validation_hints,
                transient_data_index_hints,
                note_hash_siloing_amount: 0,
                nullifier_siloing_amount: 0,
                private_log_siloing_amount: 0,
            }
        }

    pub fn get_validation_request_processor(
        self,
    ) -> PrivateValidationRequestProcessor<NOTE_HASH_PENDING_AMOUNT, NOTE_HASH_SETTLED_AMOUNT, NULLIFIER_PENDING_AMOUNT, NULLIFIER_SETTLED_AMOUNT, NULLIFIER_KEYS> {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        let note_hash_read_request_hints = self.note_hash_read_request_hints_builder.to_hints();
        let nullifier_read_request_hints = self.nullifier_read_request_hints_builder.to_hints();

        PrivateValidationRequestProcessor {
            validation_requests: previous_kernel.validation_requests,
            note_hash_read_request_hints,
            pending_note_hashes: previous_kernel.end.note_hashes,
            note_hash_tree_root: 0,
            nullifier_read_request_hints,
            pending_nullifiers: previous_kernel.end.nullifiers,
            nullifier_tree_root: 0,
            key_validation_hints: self.key_validation_hints,
            validation_requests_split_counter: previous_kernel.min_revertible_side_effect_counter,
        }
    }

    pub fn get_hints(self) -> ResetOutputHints {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        // Safety: This is only used in tests.
        unsafe {
            generate_reset_output_hints(previous_kernel, self.transient_data_index_hints)
        }
    }

    pub fn validate_with_hints(self, hints: ResetOutputHints) {
        let output = self.output.to_private_kernel_circuit_public_inputs();
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        let validation_request_processor = self.get_validation_request_processor();
        ResetOutputValidator::new(
            output,
            previous_kernel,
            validation_request_processor,
            self.transient_data_index_hints,
            self.note_hash_siloing_amount,
            self.nullifier_siloing_amount,
            self.private_log_siloing_amount,
            hints,
        )
            .validate();
    }

    pub fn validate(self) {
        let hints = self.get_hints();
        self.validate_with_hints(hints);
    }
}
