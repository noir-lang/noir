use crate::{
    address::AztecAddress, hash::{compute_tx_logs_hash, compute_tx_note_logs_hash},
    abis::{
    gas::Gas,
    accumulated_data::{
    combined_accumulated_data::CombinedAccumulatedData,
    private_accumulated_data::PrivateAccumulatedData, public_accumulated_data::PublicAccumulatedData,
    public_accumulated_data_builder::PublicAccumulatedDataBuilder
},
    call_request::CallRequest, note_hash::{NoteHash, ScopedNoteHash}, nullifier::ScopedNullifier,
    private_call_request::ScopedPrivateCallRequest, public_data_update_request::PublicDataUpdateRequest,
    log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX, DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE
},
    messaging::l2_to_l1_message::ScopedL2ToL1Message, traits::{Empty, is_empty}
};

// Builds via PrivateKernelCircuitPublicInputsBuilder:
// .finish: PrivateKernelCircuitPublicInputs.end
// .to_combined: KernelCircuitPublicInputs.end
// .split_to_public: PublicKernelCircuitPublicInputs.(end,end_non_revertible)
struct PrivateAccumulatedDataBuilder {
    new_note_hashes: BoundedVec<ScopedNoteHash, MAX_NEW_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<ScopedNullifier, MAX_NEW_NULLIFIERS_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<ScopedL2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    note_encrypted_logs_hashes: BoundedVec<NoteLogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    encrypted_logs_hashes: BoundedVec<ScopedEncryptedLogHash, MAX_ENCRYPTED_LOGS_PER_TX>,
    unencrypted_logs_hashes: BoundedVec<ScopedLogHash, MAX_UNENCRYPTED_LOGS_PER_TX>,

    private_call_stack: BoundedVec<ScopedPrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,

}

impl PrivateAccumulatedDataBuilder {
    pub fn finish(self) -> PrivateAccumulatedData {
        PrivateAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage
        }
    }

    pub fn to_combined(self, teardown_gas: Gas) -> CombinedAccumulatedData {
        // TODO(Miranda): Hash here or elsewhere?
        let note_encrypted_logs_hash = compute_tx_note_logs_hash(self.note_encrypted_logs_hashes.storage.map(|l: NoteLogHash| l.expose_to_public()));
        let encrypted_logs_hash = compute_tx_logs_hash(self.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.expose_to_public()));
        let unencrypted_logs_hash = compute_tx_logs_hash(self.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.log_hash));
        let gas_used = self.to_metered_gas_used() + Gas::tx_overhead() + teardown_gas;
        let note_encrypted_log_preimages_length = self.note_encrypted_logs_hashes.storage.fold(0, |a, b: NoteLogHash| a + b.length);
        let encrypted_log_preimages_length = self.encrypted_logs_hashes.storage.fold(0, |a, b: ScopedEncryptedLogHash| a + b.log_hash.length);
        let unencrypted_log_preimages_length = self.unencrypted_logs_hashes.storage.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);

        CombinedAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash.value),
            new_nullifiers: self.new_nullifiers.storage.map(|n: ScopedNullifier| n.nullifier.value),
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage.map(|m: ScopedL2ToL1Message| m.message.content),
            note_encrypted_logs_hash,
            encrypted_logs_hash,
            unencrypted_logs_hash,
            note_encrypted_log_preimages_length,
            encrypted_log_preimages_length,
            unencrypted_log_preimages_length,
            public_data_update_requests: [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
            gas_used
        }
    }

    pub fn to_metered_gas_used(self) -> Gas {
        let mut metered_bytes = 0;

        // note_hash_gas
        for i in 0..self.new_note_hashes.storage.len() {
            if !is_empty(self.new_note_hashes.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // nullifier_gas
        for i in 0..self.new_nullifiers.storage.len() {
            if !is_empty(self.new_nullifiers.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // l2_to_l1_msg_gas
        for i in 0..self.new_l2_to_l1_msgs.storage.len() {
            if !is_empty(self.new_l2_to_l1_msgs.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // note_encrypted_logs_hash_gas
        for i in 0..self.note_encrypted_logs_hashes.storage.len() {
            let log = self.note_encrypted_logs_hashes.get_unchecked(i);
            metered_bytes += log.length as u32;
        }

        // encrypted_logs_hash_gas
        for i in 0..self.encrypted_logs_hashes.storage.len() {
            let log = self.encrypted_logs_hashes.get_unchecked(i);
            metered_bytes += log.log_hash.length as u32;
        }

        // unencrypted_logs_hash_gas
        for i in 0..self.unencrypted_logs_hashes.storage.len() {
            let log = self.unencrypted_logs_hashes.get_unchecked(i);
            metered_bytes += log.log_hash.length as u32;
        }

        Gas::new(DA_GAS_PER_BYTE * metered_bytes, 0)
    }

    pub fn split_to_public(
        self,
        min_revertible_side_effect_counter: u32,
        teardown_gas: Gas
    ) -> (PublicAccumulatedData, PublicAccumulatedData) {
        let mut non_revertible_builder = PublicAccumulatedDataBuilder::empty();
        let mut revertible_builder = PublicAccumulatedDataBuilder::empty();
        let mut non_revertible_da_gas_used = 0;
        let mut non_revertible_l2_gas_used = 0;
        let mut revertible_da_gas_used = teardown_gas.da_gas; // pre-pay for teardown gas
        let mut revertible_l2_gas_used = teardown_gas.l2_gas;
        let DA_GAS_PER_FIELD = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            let note_hash = self.new_note_hashes.storage[i];
            let public_note_hash = note_hash.expose_to_public();
            if note_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.new_note_hashes.push(public_note_hash);
                if !is_empty(public_note_hash) {
                    non_revertible_da_gas_used += DA_GAS_PER_FIELD ;
                }
            } else {
                revertible_builder.new_note_hashes.push(public_note_hash);
                if !is_empty(public_note_hash) {
                    revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            }
        }

        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = self.new_nullifiers.storage[i];
            let public_nullifier = nullifier.expose_to_public();
            if nullifier.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.new_nullifiers.push(public_nullifier);
                if !is_empty(public_nullifier) {
                    non_revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            } else {
                revertible_builder.new_nullifiers.push(public_nullifier);
                if !is_empty(public_nullifier) {
                    revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            }
        }

        for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_TX {
            let msg = self.new_l2_to_l1_msgs.storage[i];
            if msg.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.new_l2_to_l1_msgs.push(msg.message.content);
            } else {
                revertible_builder.new_l2_to_l1_msgs.push(msg.message.content);
            }
        }

        // TODO(gas): add AVM_STARTUP_L2_GAS here
        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
            let call_stack_item = self.public_call_stack.storage[i];
            if call_stack_item.start_side_effect_counter < min_revertible_side_effect_counter {
                non_revertible_builder.public_call_stack.push(call_stack_item);
            } else {
                revertible_builder.public_call_stack.push(call_stack_item);
            }
        }

        for i in 0..MAX_NOTE_ENCRYPTED_LOGS_PER_TX {
            let note_encrypted_logs_hash = self.note_encrypted_logs_hashes.storage[i];
            let note_encrypted_logs_hash_public = note_encrypted_logs_hash.expose_to_public();
            if note_encrypted_logs_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.note_encrypted_logs_hashes.push(note_encrypted_logs_hash_public);
                non_revertible_da_gas_used += note_encrypted_logs_hash_public.length as u32 * DA_GAS_PER_BYTE;
            } else {
                revertible_builder.note_encrypted_logs_hashes.push(note_encrypted_logs_hash_public);
                revertible_da_gas_used += note_encrypted_logs_hash_public.length as u32 * DA_GAS_PER_BYTE;
            }
        }

        for i in 0..MAX_ENCRYPTED_LOGS_PER_TX {
            let encrypted_logs_hash = self.encrypted_logs_hashes.storage[i];
            let encrypted_logs_hash_public = encrypted_logs_hash.expose_to_public();
            if encrypted_logs_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.encrypted_logs_hashes.push(encrypted_logs_hash_public);
                non_revertible_da_gas_used += encrypted_logs_hash_public.length as u32 * DA_GAS_PER_BYTE;
            } else {
                revertible_builder.encrypted_logs_hashes.push(encrypted_logs_hash_public);
                revertible_da_gas_used += encrypted_logs_hash_public.length as u32 * DA_GAS_PER_BYTE;
            }
        }

        for i in 0..MAX_UNENCRYPTED_LOGS_PER_TX {
            let unencrypted_logs_hash = self.unencrypted_logs_hashes.storage[i].log_hash;
            if unencrypted_logs_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.unencrypted_logs_hashes.push(unencrypted_logs_hash);
                non_revertible_da_gas_used += unencrypted_logs_hash.length as u32 * DA_GAS_PER_BYTE;
            } else {
                revertible_builder.unencrypted_logs_hashes.push(unencrypted_logs_hash);
                revertible_da_gas_used += unencrypted_logs_hash.length as u32 * DA_GAS_PER_BYTE;
            }
        }

        revertible_builder.gas_used = Gas::new(revertible_da_gas_used, revertible_l2_gas_used);
        non_revertible_builder.gas_used = Gas::tx_overhead() + Gas::new(non_revertible_da_gas_used, non_revertible_l2_gas_used);
        (non_revertible_builder.finish(), revertible_builder.finish())
    }
}

impl Empty for PrivateAccumulatedDataBuilder {
    fn empty() -> Self {
        PrivateAccumulatedDataBuilder {
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            new_l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            private_call_stack: BoundedVec::new(),
            public_call_stack: BoundedVec::new(),
        }
    }
}

mod tests {
    use crate::{
        abis::{
        accumulated_data::private_accumulated_data_builder::PrivateAccumulatedDataBuilder, gas::Gas,
        call_request::CallRequest, caller_context::CallerContext, note_hash::NoteHash,
        nullifier::Nullifier, public_data_update_request::PublicDataUpdateRequest,
        log_hash::{LogHash, NoteLogHash, ScopedLogHash, EncryptedLogHash, ScopedEncryptedLogHash}
    },
        address::{AztecAddress, EthAddress}, messaging::l2_to_l1_message::L2ToL1Message,
        utils::arrays::array_eq, constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE}
    };

    #[test]
    unconstrained fn splits_revertible_and_non_revertible() {
        let mut builder = PrivateAccumulatedDataBuilder::empty();
        let contract_address = AztecAddress::from_field(8989);

        let min_revertible_side_effect_counter = 13;

        // Non revertible: counter < 13

        let non_revertible_note_hashes = [
            NoteHash { value: 1, counter: 1 }.scope(20, contract_address),
            NoteHash { value: 2, counter: 4 }.scope(5, contract_address)
        ];

        let non_revertible_note_logs = [NoteLogHash { value: 11, counter: 2, length: 2, note_hash_counter: 1 }];

        let non_revertible_nullifiers = [
            Nullifier { value: 10, note_hash: 1, counter: 3 }.scope(contract_address),
            Nullifier { value: 20, note_hash: 2, counter: 5 }.scope(contract_address)
        ];

        let non_revertible_l2_to_l1_messages = [
            L2ToL1Message { recipient: EthAddress::from_field(3030), content: 333333, counter: 6 }.scope(AztecAddress::from_field(9900))
        ];

        let non_revertible_public_stack = [
            CallRequest {
                hash: 1,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 6,
                end_side_effect_counter: 0
            },
            CallRequest {
                hash: 2,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 7,
                end_side_effect_counter: 0
            }
        ];

        let non_revertible_enc_log_hashes = [
            EncryptedLogHash { value: 11, counter: 9, length: 2, randomness: 4 }.scope(contract_address),
            EncryptedLogHash { value: 22, counter: 10, length: 2, randomness: 4 }.scope(contract_address)
        ];

        let non_revertible_unenc_log_hashes = [
            LogHash { value: 33, counter: 11, length: 5 }.scope(contract_address),
            LogHash { value: 44, counter: 12, length: 5 }.scope(contract_address)
        ];

        // Revertible: counter >= 13

        let revertible_note_hashes = [
            NoteHash { value: 3, counter: 13 }.scope(15, contract_address),
            NoteHash { value: 4, counter: 16 }.scope(0, contract_address)
        ];

        let revertible_note_logs = [NoteLogHash { value: 33, counter: 14, length: 2, note_hash_counter: 13 }];

        let revertible_nullifiers = [
            Nullifier { value: 30, note_hash: 3, counter: 15 }.scope(contract_address),
            Nullifier { value: 40, note_hash: 4, counter: 18 }.scope(contract_address)
        ];

        let revertible_l2_to_l1_messages = [
            L2ToL1Message { recipient: EthAddress::from_field(3030), content: 444444, counter: 19 }.scope(AztecAddress::from_field(7788))
        ];

        let revertible_public_call_stack = [
            CallRequest {
                hash: 3,
                caller_contract_address: AztecAddress::from_field(3),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 17,
                end_side_effect_counter: 0
            }
        ];

        let revertible_enc_log_hashes = [
            EncryptedLogHash { value: 55, counter: 20, length: 2, randomness: 4 }.scope(contract_address),
            EncryptedLogHash { value: 66, counter: 21, length: 2, randomness: 4 }.scope(contract_address)
        ];

        let revertible_unenc_log_hashes = [
            LogHash { value: 77, counter: 22, length: 5 }.scope(contract_address),
            LogHash { value: 88, counter: 23, length: 5 }.scope(contract_address)
        ];

        builder.new_note_hashes.extend_from_array(non_revertible_note_hashes);
        builder.new_note_hashes.extend_from_array(revertible_note_hashes);

        builder.new_nullifiers.extend_from_array(non_revertible_nullifiers);
        builder.new_nullifiers.extend_from_array(revertible_nullifiers);

        builder.new_l2_to_l1_msgs.extend_from_array(non_revertible_l2_to_l1_messages);
        builder.new_l2_to_l1_msgs.extend_from_array(revertible_l2_to_l1_messages);

        builder.public_call_stack.extend_from_array(non_revertible_public_stack);
        builder.public_call_stack.extend_from_array(revertible_public_call_stack);

        builder.note_encrypted_logs_hashes.extend_from_array(non_revertible_note_logs);
        builder.note_encrypted_logs_hashes.extend_from_array(revertible_note_logs);

        builder.encrypted_logs_hashes.extend_from_array(non_revertible_enc_log_hashes);
        builder.encrypted_logs_hashes.extend_from_array(revertible_enc_log_hashes);

        builder.unencrypted_logs_hashes.extend_from_array(non_revertible_unenc_log_hashes);
        builder.unencrypted_logs_hashes.extend_from_array(revertible_unenc_log_hashes);

        let public_non_revertible_note_logs = non_revertible_note_logs.map(|n: NoteLogHash| n.expose_to_public());
        let public_revertible_note_logs = revertible_note_logs.map(|n: NoteLogHash| n.expose_to_public());

        let (non_revertible, revertible) = builder.split_to_public(min_revertible_side_effect_counter, Gas::new(42, 17));

        assert(
            array_eq(
                non_revertible.new_note_hashes,
                [
            NoteHash { value: 1, counter: 0 },
            NoteHash { value: 2, counter: 0 }
        ]
            )
        );
        assert(
            array_eq(
                non_revertible.new_nullifiers,
                [
            Nullifier { value: 10, note_hash: 0, counter: 0 },
            Nullifier { value: 20, note_hash: 0, counter: 0 }
        ]
            )
        );
        assert(array_eq(non_revertible.new_l2_to_l1_msgs, [333333]));
        assert(array_eq(non_revertible.public_call_stack, non_revertible_public_stack));
        assert(
            array_eq(
                non_revertible.note_encrypted_logs_hashes,
                public_non_revertible_note_logs
            )
        );
        assert(
            array_eq(
                non_revertible.encrypted_logs_hashes,
                non_revertible_enc_log_hashes.map(|h: ScopedEncryptedLogHash| h.expose_to_public())
            )
        );
        assert(
            array_eq(
                non_revertible.unencrypted_logs_hashes,
                non_revertible_unenc_log_hashes.map(|h: ScopedLogHash| h.log_hash)
            )
        );

        assert(
            array_eq(
                revertible.new_note_hashes,
                [
            NoteHash { value: 3, counter: 0 },
            NoteHash { value: 4, counter: 0 }
        ]
            )
        );
        assert(
            array_eq(
                revertible.new_nullifiers,
                [
            Nullifier { value: 30, note_hash: 0, counter: 0 },
            Nullifier { value: 40, note_hash: 0, counter: 0 }
        ]
            )
        );
        assert(array_eq(revertible.new_l2_to_l1_msgs, [444444]));
        assert(array_eq(revertible.public_call_stack, revertible_public_call_stack));
        assert(
            array_eq(
                revertible.note_encrypted_logs_hashes,
                public_revertible_note_logs
            )
        );
        assert(
            array_eq(
                revertible.encrypted_logs_hashes,
                revertible_enc_log_hashes.map(|h: ScopedEncryptedLogHash| h.expose_to_public())
            )
        );
        assert(
            array_eq(
                revertible.unencrypted_logs_hashes,
                revertible_unenc_log_hashes.map(|h: ScopedLogHash| h.log_hash)
            )
        );

        assert_eq(
            revertible.gas_used, Gas::new(
            (4 * DA_BYTES_PER_FIELD
            + 6 // revertible encrypted logs len
            + 10) // revertible unencrypted logs len
            * DA_GAS_PER_BYTE,
            0
        )
            + Gas::new(42, 17)
        );

        assert_eq(
            non_revertible.gas_used, Gas::new(
            (4 * DA_BYTES_PER_FIELD
            + 6 // non-revertible encrypted logs len
            + 10) // non-revertible unencrypted logs len
            * DA_GAS_PER_BYTE,
            0
        )
            + Gas::tx_overhead()
        );
    }
}
