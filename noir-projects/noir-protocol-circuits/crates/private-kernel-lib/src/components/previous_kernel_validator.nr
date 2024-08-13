mod previous_kernel_validator_hints;

use crate::components::previous_kernel_validator::previous_kernel_validator_hints::{generate_previous_kernel_validator_hints, PreviousKernelValidatorHints};
use dep::types::{
    abis::{kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, log_hash::ScopedEncryptedLogHash},
    address::AztecAddress, traits::is_empty, utils::arrays::array_length
};

struct PreviousKernelValidator {
    previous_kernel: PrivateKernelCircuitPublicInputs,
    hints: PreviousKernelValidatorHints,
}

impl PreviousKernelValidator {
    pub fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        let hints = generate_previous_kernel_validator_hints(previous_kernel);
        PreviousKernelValidator { previous_kernel, hints }
    }

    pub fn validate_for_private_tail(self) {
        self.validate_common();
        self.validate_empty_data();
    }

    pub fn validate_for_private_tail_to_public(self) {
        self.validate_common();
        self.validate_non_empty_data();
    }

    fn validate_common(self) {
        self.validate_empty_private_call_stack();
        self.verify_empty_validation_requests();
        self.verify_sorted_siloed_values();
        self.validate_no_transient_data();
    }

    fn validate_empty_private_call_stack(self) {
        // Only need to check the first item as the kernel circuits always append items to the arrays properly.
        assert(
            is_empty(self.previous_kernel.end.private_call_stack[0]), "Private call stack must be empty when executing the tail circuit"
        );
    }

    fn validate_empty_data(self) {
        assert(
            is_empty(self.previous_kernel.end.public_call_requests[0]), "Public call stack must be empty when executing the tail circuit"
        );
        assert(
            is_empty(self.previous_kernel.public_teardown_call_request), "Public teardown call request must be empty when executing the tail circuit"
        );

        if self.previous_kernel.validation_requests.split_counter.is_some() {
            // Even when min_revertible_side_effect_counter could be non-zero in a pure private tx.
            // The split counter must be 0 to ensure that all the transient data are squashed.
            assert_eq(
                self.previous_kernel.validation_requests.split_counter.unwrap_unchecked(), 0, "split_counter must be 0 for pure private tx"
            );
        }
    }

    fn validate_non_empty_data(self) {
        assert(
            !is_empty(self.previous_kernel.end.public_call_requests[0])
            | !is_empty(self.previous_kernel.public_teardown_call_request), "Must have public calls when exporting public kernel data from the tail circuit"
        );

        assert(
            self.previous_kernel.min_revertible_side_effect_counter != 0, "min_revertible_side_effect_counter must not be 0"
        );

        if self.previous_kernel.validation_requests.split_counter.is_some() {
            assert_eq(
                self.previous_kernel.validation_requests.split_counter.unwrap_unchecked(), self.previous_kernel.min_revertible_side_effect_counter, "split_counter does not match min_revertible_side_effect_counter"
            );
        }
    }

    fn verify_empty_validation_requests(self) {
        assert(
            is_empty(self.previous_kernel.validation_requests.note_hash_read_requests[0]), "Non empty note hash read requests"
        );
        assert(
            is_empty(self.previous_kernel.validation_requests.nullifier_read_requests[0]), "Non empty nullifier read requests"
        );
        assert(
            is_empty(self.previous_kernel.validation_requests.scoped_key_validation_requests_and_generators[0]), "Non empty key validation requests"
        );
    }

    fn verify_sorted_siloed_values(self) {
        // Check that the data are already siloed and/or sorted in the reset circuit.
        // Any unprocessed data added after the last reset with siloing was called should be caught here.

        let num_note_hashes = array_length(self.previous_kernel.end.note_hashes);
        if num_note_hashes != 0 {
            let note_hash = self.previous_kernel.end.note_hashes[num_note_hashes - 1];
            assert_eq(
                note_hash.contract_address, AztecAddress::zero(), "note hashes have not been siloed in a reset"
            );
        }

        let num_nullifiers = array_length(self.previous_kernel.end.nullifiers);
        let nullifier = self.previous_kernel.end.nullifiers[num_nullifiers - 1]; // - 1 without checking because there's at least 1 nullifier.
        assert_eq(
            nullifier.contract_address, AztecAddress::zero(), "nullifiers have not been siloed in a reset"
        );

        // Note logs are not siloed, but they are sorted and their note_hash_counter should've been set to 0 in the reset circuit.
        let num_note_logs = array_length(self.previous_kernel.end.note_encrypted_logs_hashes);
        if num_note_logs != 0 {
            let note_log = self.previous_kernel.end.note_encrypted_logs_hashes[num_note_logs - 1];
            assert_eq(note_log.note_hash_counter, 0, "note logs have not been sorted in a reset");
        }

        // We need to check the entire array because randomness can be 0 for encrypted logs if the app wants to reveal the actual contract address.
        assert(
            self.previous_kernel.end.encrypted_logs_hashes.all(|h: ScopedEncryptedLogHash| h.log_hash.randomness == 0), "encrypted logs have not been siloed in a reset"
        );
    }

    fn validate_no_transient_data(self) {
        let nullifiers = self.previous_kernel.end.nullifiers;
        let note_hashes = self.previous_kernel.end.note_hashes;
        let note_hash_indexes_for_nullifiers = self.hints.note_hash_indexes_for_nullifiers;
        for i in 0..nullifiers.len() {
            let nullifier = nullifiers[i];
            let nullified_note_hash = nullifier.nullifier.note_hash;
            if nullified_note_hash != 0 {
                let note_hash = note_hashes[note_hash_indexes_for_nullifiers[i]];
                assert_eq(
                    note_hash.value(), nullified_note_hash, "Hinted siloed note hash does not match nullified note hash"
                );
                assert(
                    note_hash.counter() < nullifier.counter(), "Cannot link a note hash emitted after a nullifier"
                );
            }
        }
    }
}
