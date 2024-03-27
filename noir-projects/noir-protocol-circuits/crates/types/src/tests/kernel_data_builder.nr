use crate::{
    abis::{
    call_context::CallContext, call_request::{CallerContext, CallRequest},
    combined_constant_data::CombinedConstantData,
    accumulated_data::{CombinedAccumulatedDataBuilder, AccumulatedNonRevertibleDataBuilder, AccumulatedRevertibleDataBuilder},
    kernel_circuit_public_inputs::{
    PrivateKernelInnerCircuitPublicInputs, PrivateKernelTailCircuitPublicInputs,
    PublicKernelCircuitPublicInputs, RollupKernelCircuitPublicInputs
},
    kernel_data::{PrivateKernelInnerData, PublicKernelData, RollupKernelData},
    max_block_number::MaxBlockNumber, public_data_read::PublicDataRead,
    public_data_update_request::PublicDataUpdateRequest, read_request::ReadRequestContext,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    validation_requests::ValidationRequestsBuilder
},
    address::{AztecAddress, EthAddress}, header::Header, hash::silo_nullifier,
    mocked::{AggregationObject, Proof, VerificationKey},
    tests::{fixtures, testing_harness::build_tx_context}, transaction::tx_context::TxContext
};
use crate::constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, VK_TREE_HEIGHT
};
use dep::std::unsafe;

struct PreviousKernelDataBuilder {
    contract_address: AztecAddress,
    storage_contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    validation_requests: ValidationRequestsBuilder,
    end: CombinedAccumulatedDataBuilder,
    end_non_revertible: AccumulatedNonRevertibleDataBuilder,
    historical_header: Header,
    tx_context: TxContext,
    is_private: bool,
    proof: Proof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
    sideffect_counter: u32,
    min_revertible_side_effect_counter: u32,
}

impl PreviousKernelDataBuilder {
    pub fn new(is_for_public: bool) -> Self {
        let mut end: CombinedAccumulatedDataBuilder = dep::std::unsafe::zeroed();
        let mut end_non_revertible: AccumulatedNonRevertibleDataBuilder = dep::std::unsafe::zeroed();
        // 0th nullifier must be non-zero.
        let tx_nullifier = SideEffectLinkedToNoteHash { value: 321, note_hash: 0, counter: 0 };
        // but the public kernels expect this first nullifier to have been partitioned into
        // the non-revertible set

        if is_for_public == true {
            end_non_revertible.new_nullifiers.push(tx_nullifier);
        } else {
            end.new_nullifiers.push(tx_nullifier);
        }

        let tx_context = build_tx_context();

        PreviousKernelDataBuilder {
            contract_address: fixtures::contracts::parent_contract.address,
            storage_contract_address: fixtures::contracts::parent_contract.address,
            portal_contract_address: fixtures::contracts::parent_contract.portal_contract_address,
            validation_requests: unsafe::zeroed(),
            end,
            end_non_revertible,
            historical_header: fixtures::HEADER,
            tx_context,
            is_private: true,
            proof: Proof {},
            vk: VerificationKey {},
            vk_index: 0,
            vk_path: [0; VK_TREE_HEIGHT],
            sideffect_counter: 2,
            min_revertible_side_effect_counter: 2
        }
    }

    pub fn is_public(&mut self) -> Self {
        self.is_private = false;
        *self
    }

    pub fn set_max_block_number(&mut self, max_block_number: u32) {
        self.validation_requests.max_block_number = MaxBlockNumber::new(max_block_number);
    }

    pub fn append_public_data_update_requests(&mut self, num_updates: u64) {
        let value_offset = self.end.public_data_update_requests.len();
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            if i < num_updates {
                let update_request = PublicDataUpdateRequest {
                    // The default leaf index is its index + 23.
                    leaf_slot: (value_offset + i + 23) as Field,
                    // The default value is its index + 678.
                    new_value: (value_offset + i + 678) as Field
                };
                self.end.public_data_update_requests.push(update_request);
            }
        }
    }

    pub fn append_public_data_read_requests(&mut self, num_reads: u64) {
        let value_offset = self.validation_requests.public_data_reads.len();
        for i in 0..MAX_PUBLIC_DATA_READS_PER_TX {
            if i < num_reads {
                let read_request = PublicDataRead {
                    // The default leaf index is its index + 34.
                    leaf_slot: (value_offset + i + 34) as Field,
                    // The default value is its index + 5566.
                    value: (value_offset + i + 5566) as Field
                };
                self.validation_requests.public_data_reads.push(read_request);
            }
        }
    }

    fn next_sideffect_counter(&mut self) -> u32 {
        let counter = self.sideffect_counter;
        self.sideffect_counter += 1;
        counter
    }

    pub fn add_read_request_for_transient_commitment(&mut self, commitment_index: u64) -> u64 {
        let new_read_request_index = self.validation_requests.note_hash_read_requests.len();
        let commitment = self.end.new_note_hashes.get(commitment_index);
        let read_request = SideEffect { value: commitment.value, counter: self.next_sideffect_counter() };
        self.validation_requests.note_hash_read_requests.push(read_request);
        new_read_request_index
    }

    pub fn append_new_note_hashes(&mut self, num_new_note_hashes: u64) {
        let mocked_value_offset = self.end.new_note_hashes.len() + 1;
        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            if i < num_new_note_hashes {
                // The default value is its index + 1.
                self.end.new_note_hashes.push(
                    SideEffect { value: (i + mocked_value_offset) as Field, counter: self.next_sideffect_counter() }
                );
            }
        }
    }

    fn get_mock_nullifier_value(_self: Self, nullifier_index: u64) -> Field {
        let value_offset = 5678;
        value_offset + nullifier_index as Field
    }

    fn get_mock_nullifier_value_non_revertible(_self: Self, nullifier_index: u64) -> Field {
        let value_offset = 987;
        value_offset + nullifier_index as Field
    }

    pub fn add_nullifier(&mut self, unsiloed_nullifier: Field) {
        let value = silo_nullifier(self.storage_contract_address, unsiloed_nullifier);
        self.end.new_nullifiers.push(SideEffectLinkedToNoteHash { value, note_hash: 0, counter: self.next_sideffect_counter() });
    }

    pub fn add_nullifier_non_revertible(&mut self, unsiloed_nullifier: Field) {
        let value = silo_nullifier(self.storage_contract_address, unsiloed_nullifier);
        self.end_non_revertible.new_nullifiers.push(SideEffectLinkedToNoteHash { value, note_hash: 0, counter: self.next_sideffect_counter() });
    }

    pub fn append_new_nullifiers_from_private(&mut self, num_extra_nullifier: u64) {
        // in private kernel, the nullifiers have not yet been partitioned
        // (that is part of the job of the private kernel tail)
        // so the tx nullifier is in `end`
        let index_offset = self.end.new_nullifiers.len();
        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if i < num_extra_nullifier {
                let mock_value = self.get_mock_nullifier_value(index_offset + i);
                self.add_nullifier(mock_value);
            }
        }
    }

    pub fn append_new_nullifiers_from_public(&mut self, num_extra_nullifier: u64) {
        let index_offset = self.end.new_nullifiers.len();
        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if i < num_extra_nullifier {
                let mock_value = self.get_mock_nullifier_value(index_offset + i);
                self.add_nullifier(mock_value);
            }
        }
    }

    pub fn append_new_nullifiers_non_revertible_from_public(&mut self, num_extra_nullifier: u64) {
        let index_offset = self.end_non_revertible.new_nullifiers.len();
        for i in 0..MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX {
            if i < num_extra_nullifier {
                let mock_value = self.get_mock_nullifier_value_non_revertible(index_offset + i);
                self.add_nullifier_non_revertible(mock_value);
            }
        }
    }

    pub fn add_read_request_for_pending_nullifier(&mut self, nullifier_index: u64) -> u64 {
        let read_request_index = self.validation_requests.nullifier_read_requests.len();
        let unsiloed_nullifier = self.get_mock_nullifier_value(nullifier_index);
        let read_request = ReadRequestContext {
            value: unsiloed_nullifier,
            counter: self.next_sideffect_counter(),
            contract_address: self.storage_contract_address
        };
        self.validation_requests.nullifier_read_requests.push(read_request);
        read_request_index
    }

    pub fn add_read_request_for_pending_non_revertible_nullifier(&mut self, nullifier_index: u64) -> u64 {
        let read_request_index = self.validation_requests.nullifier_read_requests.len();
        let unsiloed_nullifier = self.get_mock_nullifier_value_non_revertible(nullifier_index);
        let read_request = ReadRequestContext {
            value: unsiloed_nullifier,
            counter: self.next_sideffect_counter(),
            contract_address: self.storage_contract_address
        };
        self.validation_requests.nullifier_read_requests.push(read_request);
        read_request_index
    }

    pub fn add_non_existent_read_request_for_nullifier(&mut self, unsiloed_nullifier: Field) {
        let read_request = ReadRequestContext {
            value: unsiloed_nullifier,
            counter: self.next_sideffect_counter(),
            contract_address: self.storage_contract_address
        };
        self.validation_requests.nullifier_non_existent_read_requests.push(read_request);
    }

    // snapshot the side effects
    // this is useful in the private tail circuit to test side effect splitting
    pub fn capture_min_revertible_side_effect_counter(&mut self) {
        self.min_revertible_side_effect_counter = self.sideffect_counter;
    }

    pub fn set_encrypted_logs(&mut self, hash: Field, preimages_length: Field) {
        self.end.encrypted_logs_hash = hash;
        self.end.encrypted_log_preimages_length = preimages_length;
    }

    pub fn set_unencrypted_logs(&mut self, hash: Field, preimages_length: Field) {
        self.end.unencrypted_logs_hash = hash;
        self.end.unencrypted_log_preimages_length = preimages_length;
    }

    pub fn push_private_call_request(&mut self, hash: Field, is_delegate_call: bool) {
        let call_stack_item = self.generate_call_request(hash, is_delegate_call);
        self.end.private_call_stack.push(call_stack_item);
    }

    pub fn push_public_call_request(&mut self, hash: Field, is_delegate_call: bool) {
        let call_stack_item = self.generate_call_request(hash, is_delegate_call);
        self.end.public_call_stack.push(call_stack_item);
    }

    pub fn push_public_call_request_non_revertible(&mut self, hash: Field, is_delegate_call: bool) {
        let call_stack_item = self.generate_call_request(hash, is_delegate_call);
        self.end_non_revertible.public_call_stack.push(call_stack_item);
    }

    fn generate_call_request(&mut self, hash: Field, is_delegate_call: bool) -> CallRequest {
        let mut caller_context = CallerContext::empty();
        if is_delegate_call {
            caller_context.msg_sender = fixtures::MSG_SENDER;
            caller_context.storage_contract_address = self.contract_address;
        }
        let counter = self.next_sideffect_counter();
        let end_counter = counter + 10;
        self.sideffect_counter = end_counter;
        CallRequest {
            hash,
            caller_contract_address: self.contract_address,
            caller_context,
            start_side_effect_counter: counter,
            end_side_effect_counter: end_counter
        }
    }

    pub fn to_private_kernel_inner_data(self) -> PrivateKernelInnerData {
        let public_inputs = PrivateKernelInnerCircuitPublicInputs {
            aggregation_object: AggregationObject {},
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            end: self.end.finish(),
            validation_requests: self.validation_requests.finish(),
            constants: CombinedConstantData { historical_header: self.historical_header, tx_context: self.tx_context },
            is_private: self.is_private
        };
        PrivateKernelInnerData { public_inputs, proof: self.proof, vk: self.vk, vk_index: self.vk_index, vk_path: self.vk_path }
    }

    pub fn to_public_kernel_data(self) -> PublicKernelData {
        let end_non_revertible = self.end_non_revertible.to_public();
        let end = self.end.to_public_accumulated_revertible_data();
        let public_inputs = PublicKernelCircuitPublicInputs {
            aggregation_object: AggregationObject {},
            end_non_revertible,
            end,
            validation_requests: self.validation_requests.finish(),
            rollup_validation_requests: self.validation_requests.to_rollup(),
            constants: CombinedConstantData { historical_header: self.historical_header, tx_context: self.tx_context },
            needs_setup: end_non_revertible.needs_setup(),
            needs_app_logic: end.needs_app_logic(),
            needs_teardown: end_non_revertible.needs_teardown()
        };

        PublicKernelData { public_inputs, proof: self.proof, vk: self.vk, vk_index: self.vk_index, vk_path: self.vk_path }
    }

    pub fn to_rollup_kernel_data(self) -> RollupKernelData {
        let public_inputs = RollupKernelCircuitPublicInputs {
            aggregation_object: AggregationObject {},
            end: self.end.finish(),
            constants: CombinedConstantData { historical_header: self.historical_header, tx_context: self.tx_context },
            rollup_validation_requests: self.validation_requests.to_rollup()
        };

        RollupKernelData { public_inputs, proof: self.proof, vk: self.vk, vk_index: self.vk_index, vk_path: self.vk_path }
    }
}
