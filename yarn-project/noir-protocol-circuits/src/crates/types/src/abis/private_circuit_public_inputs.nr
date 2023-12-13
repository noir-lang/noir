use crate::{
    abis::{
        call_context::CallContext,
        block_header::BlockHeader,
    },
    contrakt::deployment_data::ContractDeploymentData,
    hash::{
        NUM_FIELDS_PER_SHA256,
        pedersen_hash,
    },
    utils::bounded_vec::BoundedVec,
};
use crate::constants::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PENDING_READ_REQUESTS_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        RETURN_VALUES_LENGTH,
        PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
        PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
        GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS,
};

struct PrivateCircuitPublicInputs {
    call_context: CallContext,
    
    args_hash: Field,
    return_values: [Field; RETURN_VALUES_LENGTH],

    read_requests: [Field; MAX_READ_REQUESTS_PER_CALL],
    pending_read_requests: [Field; MAX_PENDING_READ_REQUESTS_PER_CALL],

    new_commitments: [Field; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers: [Field; MAX_NEW_NULLIFIERS_PER_CALL],
    nullified_commitments: [Field; MAX_NEW_NULLIFIERS_PER_CALL],

    private_call_stack_hashes: [Field; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack_hashes: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    block_header: BlockHeader,

    contract_deployment_data: ContractDeploymentData,

    chain_id: Field,
    version: Field,
}

impl PrivateCircuitPublicInputs {
    fn hash(self) -> Field {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH> = BoundedVec::new(0); 
        fields.push(self.call_context.hash());
        fields.push(self.args_hash);
        fields.push_array(self.return_values);
        fields.push_array(self.read_requests);
        fields.push_array(self.pending_read_requests);
        fields.push_array(self.new_commitments);
        fields.push_array(self.new_nullifiers);
        fields.push_array(self.nullified_commitments);
        fields.push_array(self.private_call_stack_hashes);
        fields.push_array(self.public_call_stack_hashes);
        fields.push_array(self.new_l2_to_l1_msgs);
        fields.push_array(self.encrypted_logs_hash);
        fields.push_array(self.unencrypted_logs_hash);
        fields.push(self.encrypted_log_preimages_length);
        fields.push(self.unencrypted_log_preimages_length);
        fields.push_array(self.block_header.serialize());
        fields.push(self.contract_deployment_data.hash());
        fields.push(self.chain_id);
        fields.push(self.version);

        assert_eq(fields.len(), PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH, "Incorrect number of input fields when hashing PrivateCircuitPublicInputs");

        pedersen_hash(fields.storage, GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS)
    }

    fn serialize(self) -> [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields.push_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.push_array(self.return_values);
        fields.push_array(self.read_requests);
        fields.push_array(self.pending_read_requests);
        fields.push_array(self.new_commitments);
        fields.push_array(self.new_nullifiers);
        fields.push_array(self.private_call_stack_hashes);
        fields.push_array(self.public_call_stack_hashes);
        fields.push_array(self.new_l2_to_l1_msgs);
        fields.push_array(self.encrypted_logs_hash);
        fields.push_array(self.unencrypted_logs_hash);
        fields.push(self.encrypted_log_preimages_length);
        fields.push(self.unencrypted_log_preimages_length);
        fields.push_array(self.block_header.serialize());
        fields.push_array(self.contract_deployment_data.serialize());
        fields.push(self.chain_id);
        fields.push(self.version);
        fields.storage
    }
}
