mod previous_kernel_validator_hints;

use dep::types::{
    abis::{private_kernel_data::PrivateKernelData, side_effect::{Ordered, OrderedValue}},
    address::AztecAddress,
    traits::is_empty,
    utils::arrays::array_length,
};
use previous_kernel_validator_hints::{
    generate_previous_kernel_validator_hints, PreviousKernelValidatorHints,
};

pub struct PreviousKernelValidator {
    previous_kernel: PrivateKernelData,
    hints: PreviousKernelValidatorHints,
}

impl PreviousKernelValidator {
    pub fn new(previous_kernel: PrivateKernelData) -> Self {
        // Safety: the below hints are constrained by the following methods. See private_kernel_inner for use.
        let hints =
            unsafe { generate_previous_kernel_validator_hints(previous_kernel.public_inputs) };
        PreviousKernelValidator { previous_kernel, hints }
    }

    pub fn validate_proof<let N: u32>(self: Self, allowed_indices: [u32; N]) {
        if !dep::std::runtime::is_unconstrained() {
            self.previous_kernel.validate_in_vk_tree(allowed_indices);
        }
    }

    pub fn validate_for_private_tail(self) {
        assert(
            self.previous_kernel.public_inputs.is_private_only,
            "Must be private only to be processed in tail",
        );
        self.validate_common();
        self.validate_empty_data();
        self.validate_first_nullifier(false);
    }

    pub fn validate_for_private_tail_to_public(self) {
        assert(
            !self.previous_kernel.public_inputs.is_private_only,
            "Must not be private only to be processed in tail to public",
        );
        self.validate_common();
        self.validate_non_empty_data();
        self.validate_first_nullifier(true);
    }

    fn validate_first_nullifier(self, tx_can_revert: bool) {
        let first_nullifier = self.previous_kernel.public_inputs.end.nullifiers[0];
        assert_eq(
            first_nullifier.value(),
            self.previous_kernel.public_inputs.claimed_first_nullifier,
            "First nullifier claim was not satisfied",
        );
        if tx_can_revert {
            assert(
                first_nullifier.nullifier.counter()
                    < self.previous_kernel.public_inputs.min_revertible_side_effect_counter,
                "First nullifier must be non revertible",
            );
        }
    }

    fn validate_common(self) {
        self.validate_fee_payer();
        self.validate_empty_private_call_stack();
        self.verify_empty_validation_requests();
        self.verify_siloed_values();
        self.verify_no_transient_data();
    }

    fn validate_fee_payer(self) {
        assert(!is_empty(self.previous_kernel.public_inputs.fee_payer), "Fee payer can't be empty");
    }

    fn validate_empty_private_call_stack(self) {
        // Only need to check the first item as the kernel circuits always append items to the arrays properly.
        assert(
            is_empty(self.previous_kernel.public_inputs.end.private_call_stack[0]),
            "Private call stack must be empty when executing the tail circuit",
        );
    }

    fn validate_empty_data(self) {
        assert(
            is_empty(
                self.previous_kernel.public_inputs.end.public_call_requests[0],
            ),
            "Public call stack must be empty when executing the tail circuit",
        );
        assert(
            is_empty(
                self.previous_kernel.public_inputs.public_teardown_call_request,
            ),
            "Public teardown call request must be empty when executing the tail circuit",
        );

        if self.previous_kernel.public_inputs.validation_requests.split_counter.is_some() {
            // Even when min_revertible_side_effect_counter could be non-zero in a pure private tx.
            // The split counter must be 0 to ensure that all the transient data are squashed.
            assert_eq(
                self
                    .previous_kernel
                    .public_inputs
                    .validation_requests
                    .split_counter
                    .unwrap_unchecked(),
                0,
                "split_counter must be 0 for pure private tx",
            );
        }
    }

    fn validate_non_empty_data(self) {
        assert(
            !is_empty(
                self.previous_kernel.public_inputs.end.public_call_requests[0],
            )
                | !is_empty(
                    self.previous_kernel.public_inputs.public_teardown_call_request,
                ),
            "Must have public calls when exporting public kernel data from the tail circuit",
        );

        assert(
            self.previous_kernel.public_inputs.min_revertible_side_effect_counter != 0,
            "min_revertible_side_effect_counter must not be 0",
        );

        if self.previous_kernel.public_inputs.validation_requests.split_counter.is_some() {
            assert_eq(
                self
                    .previous_kernel
                    .public_inputs
                    .validation_requests
                    .split_counter
                    .unwrap_unchecked(),
                self.previous_kernel.public_inputs.min_revertible_side_effect_counter,
                "split_counter does not match min_revertible_side_effect_counter",
            );
        }
    }

    fn verify_empty_validation_requests(self) {
        assert(
            is_empty(
                self.previous_kernel.public_inputs.validation_requests.note_hash_read_requests[0],
            ),
            "Non empty note hash read requests",
        );
        assert(
            is_empty(
                self.previous_kernel.public_inputs.validation_requests.nullifier_read_requests[0],
            ),
            "Non empty nullifier read requests",
        );
        assert(
            is_empty(
                self
                    .previous_kernel
                    .public_inputs
                    .validation_requests
                    .scoped_key_validation_requests_and_generators[0],
            ),
            "Non empty key validation requests",
        );
    }

    // Ensure that the data has been properly siloed in the reset circuit.
    fn verify_siloed_values(self) {
        // note_hashes
        let num_note_hashes = array_length(self.previous_kernel.public_inputs.end.note_hashes);
        if num_note_hashes != 0 {
            let note_hash = self.previous_kernel.public_inputs.end.note_hashes[num_note_hashes - 1];
            assert_eq(
                note_hash.contract_address,
                AztecAddress::zero(),
                "note hashes have not been siloed in a reset",
            );
        }

        // nullifiers
        let num_nullifiers = array_length(self.previous_kernel.public_inputs.end.nullifiers);
        let nullifier = self.previous_kernel.public_inputs.end.nullifiers[num_nullifiers - 1]; // - 1 without checking because there's at least 1 nullifier.
        assert_eq(
            nullifier.contract_address,
            AztecAddress::zero(),
            "nullifiers have not been siloed in a reset",
        );

        // private_logs
        let num_private_logs = array_length(self.previous_kernel.public_inputs.end.private_logs);
        if num_private_logs != 0 {
            let private_log =
                self.previous_kernel.public_inputs.end.private_logs[num_private_logs - 1];
            assert_eq(
                private_log.contract_address,
                AztecAddress::zero(),
                "private logs have not been siloed in a reset",
            );
        }
    }

    fn verify_no_transient_data(self) {
        let nullifiers = self.previous_kernel.public_inputs.end.nullifiers;
        let note_hashes = self.previous_kernel.public_inputs.end.note_hashes;
        let note_hash_indexes_for_nullifiers = self.hints.note_hash_indexes_for_nullifiers;
        for i in 0..nullifiers.len() {
            let nullifier = nullifiers[i];
            let nullified_note_hash = nullifier.nullifier.note_hash;
            if nullified_note_hash != 0 {
                let note_hash = note_hashes[note_hash_indexes_for_nullifiers[i]];
                assert_eq(
                    note_hash.value(),
                    nullified_note_hash,
                    "Hinted siloed note hash does not match nullified note hash",
                );
                assert(
                    note_hash.counter() < nullifier.counter(),
                    "Cannot link a note hash emitted after a nullifier",
                );
                // No need to verify logs linked to a note hash are squashed.
                // When a note hash is squashed, all associated logs are guaranteed to be removed.
                // See reset-kernel-lib/src/reset/transient_data.nr for details.
            }
        }
    }
}
