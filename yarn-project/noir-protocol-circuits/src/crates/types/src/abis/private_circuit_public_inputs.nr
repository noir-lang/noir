use crate::{
    abis::{
        call_context::CallContext,
        nullifier_key_validation_request::NullifierKeyValidationRequest,
        side_effect::{
            SideEffect,
            SideEffectLinkedToNoteHash,
        },
    },
    constants::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        NUM_FIELDS_PER_SHA256,
        RETURN_VALUES_LENGTH,
        PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
        GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS,
    },
    contrakt::deployment_data::ContractDeploymentData,
    header::Header,
    hash::pedersen_hash,
    traits::{
        Deserialize,
        Hash,
        Serialize,
    },
    utils::reader::Reader,
};

struct PrivateCircuitPublicInputs {
    call_context: CallContext,

    args_hash: Field,
    return_values: [Field; RETURN_VALUES_LENGTH],

    meta_hwm: u32,

    read_requests: [SideEffect; MAX_READ_REQUESTS_PER_CALL],
    nullifier_key_validation_requests: [NullifierKeyValidationRequest; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL],

    new_commitments: [SideEffect; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_CALL],
    private_call_stack_hashes: [Field; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack_hashes: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
    end_side_effect_counter : u32,

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    contract_deployment_data: ContractDeploymentData,

    // Note: The following 2 values are not redundant to the values in self.historical_header.global_variables because
    // they can be different in case of a protocol upgrade. In such a situation we could be using header from a block
    // before the upgrade took place but be using the updated protocol to execute and prove the transaction.
    chain_id: Field,
    version: Field,
}

impl Eq for PrivateCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        self.call_context.eq(other.call_context) &
        self.args_hash.eq(other.args_hash) &
        (self.return_values == other.return_values) &
        (self.read_requests == other.read_requests) &
        (self.nullifier_key_validation_requests == other.nullifier_key_validation_requests) &
        (self.new_commitments == other.new_commitments) &
        (self.new_nullifiers == other.new_nullifiers) &
        (self.private_call_stack_hashes == other.private_call_stack_hashes) &
        (self.public_call_stack_hashes == other.public_call_stack_hashes) &
        (self.new_l2_to_l1_msgs == other.new_l2_to_l1_msgs) &
        (self.end_side_effect_counter == other.end_side_effect_counter) &
        (self.encrypted_logs_hash == other.encrypted_logs_hash) &
        (self.unencrypted_logs_hash == other.unencrypted_logs_hash) &
        (self.encrypted_log_preimages_length == other.encrypted_log_preimages_length) &
        (self.unencrypted_log_preimages_length == other.unencrypted_log_preimages_length) &
        self.historical_header.eq(other.historical_header) &
        self.contract_deployment_data.eq(other.contract_deployment_data) &
        self.chain_id.eq(other.chain_id) &
        self.version.eq(other.version)
    }
}

impl Serialize<PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateCircuitPublicInputs {
    fn serialize(self) -> [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0);
        fields.extend_from_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.extend_from_array(self.return_values);

        fields.push(self.meta_hwm as Field);

        for i in 0..MAX_READ_REQUESTS_PER_CALL{
            fields.extend_from_array(self.read_requests[i].serialize());
        }
        for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL{
            fields.extend_from_array(self.nullifier_key_validation_requests[i].serialize());
        }
        for i in 0..MAX_NEW_COMMITMENTS_PER_CALL{
            fields.extend_from_array(self.new_commitments[i].serialize());
        }
        for i in 0..MAX_NEW_NULLIFIERS_PER_CALL{
            fields.extend_from_array(self.new_nullifiers[i].serialize());
        }
        fields.extend_from_array(self.private_call_stack_hashes);
        fields.extend_from_array(self.public_call_stack_hashes);
        fields.extend_from_array(self.new_l2_to_l1_msgs);
        fields.push(self.end_side_effect_counter as Field);
        fields.extend_from_array(self.encrypted_logs_hash);
        fields.extend_from_array(self.unencrypted_logs_hash);
        fields.push(self.encrypted_log_preimages_length);
        fields.push(self.unencrypted_log_preimages_length);
        fields.extend_from_array(self.historical_header.serialize());
        fields.extend_from_array(self.contract_deployment_data.serialize());
        fields.push(self.chain_id);
        fields.push(self.version);

        assert_eq(fields.len(), PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH);

        fields.storage
    }
}

impl Deserialize<PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateCircuitPublicInputs {
    fn deserialize(serialized: [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);
        let inputs = Self {
            call_context: reader.read_struct(CallContext::deserialize),
            args_hash: reader.read(),
            return_values: reader.read_array([0; RETURN_VALUES_LENGTH]),
            meta_hwm: reader.read() as u32,
            read_requests: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_READ_REQUESTS_PER_CALL]),
            nullifier_key_validation_requests: reader.read_struct_array(NullifierKeyValidationRequest::deserialize, [NullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL]),
            new_commitments: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_NEW_COMMITMENTS_PER_CALL]),
            new_nullifiers: reader.read_struct_array(SideEffectLinkedToNoteHash::deserialize, [SideEffectLinkedToNoteHash::empty(); MAX_NEW_NULLIFIERS_PER_CALL]),
            private_call_stack_hashes: reader.read_array([0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL]),
            public_call_stack_hashes: reader.read_array([0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]),
            new_l2_to_l1_msgs: reader.read_array([0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL]),
            end_side_effect_counter: reader.read() as u32,
            encrypted_logs_hash: reader.read_array([0; NUM_FIELDS_PER_SHA256]),
            unencrypted_logs_hash: reader.read_array([0; NUM_FIELDS_PER_SHA256]),
            encrypted_log_preimages_length: reader.read(),
            unencrypted_log_preimages_length: reader.read(),
            historical_header: reader.read_struct(Header::deserialize),
            contract_deployment_data: reader.read_struct(ContractDeploymentData::deserialize),
            chain_id: reader.read(),
            version: reader.read(),
        };

        reader.finish();
        inputs
    }
}

impl Hash for PrivateCircuitPublicInputs {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS)
    }
}

#[test]
fn serialization_of_empty() {
    let pcpi: PrivateCircuitPublicInputs = dep::std::unsafe::zeroed();
    let serialized = pcpi.serialize();
    let deserialized = PrivateCircuitPublicInputs::deserialize(serialized);
    assert(pcpi.eq(deserialized));
}

#[test]
fn empty_hash() {
    let inputs: PrivateCircuitPublicInputs = dep::std::unsafe::zeroed();
    let hash = inputs.hash();

    // Value from private_circuit_public_inputs.test.ts "computes empty item hash" test
    assert_eq(hash, 0x1304e6c42e3c53fc7c918cbf1ea70333b4f214726c0784cf9878cd641967dab1);
}
