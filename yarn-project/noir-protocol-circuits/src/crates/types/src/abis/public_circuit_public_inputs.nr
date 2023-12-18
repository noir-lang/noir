use crate::constants::{
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    RETURN_VALUES_LENGTH,
    GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH,
};
use crate::{
    abis::{
        call_context::CallContext,
        block_header::BlockHeader,
    },
    address::AztecAddress,
    contrakt::{
        storage_read::StorageRead,
        storage_update_request::StorageUpdateRequest,
    },
    hash::{NUM_FIELDS_PER_SHA256},
    utils::bounded_vec::BoundedVec,
};

struct PublicCircuitPublicInputs{
    call_context: CallContext,

    args_hash: Field,
    return_values: [Field; RETURN_VALUES_LENGTH],

    contract_storage_update_requests: [StorageUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
    contract_storage_reads: [StorageRead; MAX_PUBLIC_DATA_READS_PER_CALL],

    public_call_stack_hashes: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_commitments: [Field; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers: [Field; MAX_NEW_NULLIFIERS_PER_CALL],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],

    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    unencrypted_log_preimages_length: Field,

    block_header: BlockHeader,

    prover_address: AztecAddress,
}

impl PublicCircuitPublicInputs{
    fn hash(self) -> Field {      
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595) have mercy por favor  
        let mut inputs: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH> = BoundedVec::new(0);
        inputs.push(self.call_context.hash());
        inputs.push(self.args_hash);
        inputs.push_array(self.return_values);
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            inputs.push(self.contract_storage_update_requests[i].hash());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            inputs.push(self.contract_storage_reads[i].hash());
        }
        inputs.push_array(self.public_call_stack_hashes);
        inputs.push_array(self.new_commitments);
        inputs.push_array(self.new_nullifiers);
        inputs.push_array(self.new_l2_to_l1_msgs);
        inputs.push_array(self.unencrypted_logs_hash);
        inputs.push(self.unencrypted_log_preimages_length);
        inputs.push_array(self.block_header.serialize());
        inputs.push(self.prover_address.to_field());

        assert_eq(inputs.len(), PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH, "Incorrect number of input fields when hashing PublicCircuitPublicInputs");

        dep::std::hash::pedersen_hash_with_separator(inputs.storage, GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS)
    }

    pub fn serialize(self) -> [Field; PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields.push_array(self.call_context.serialize());   
        fields.push(self.args_hash);
        fields.push_array(self.return_values);
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            fields.push_array(self.contract_storage_update_requests[i].serialize());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            fields.push_array(self.contract_storage_reads[i].serialize());
        }
        fields.push_array(self.public_call_stack_hashes);
        fields.push_array(self.new_commitments);
        fields.push_array(self.new_nullifiers);
        fields.push_array(self.new_l2_to_l1_msgs);
        fields.push_array(self.unencrypted_logs_hash);
        fields.push(self.unencrypted_log_preimages_length);
        fields.push_array(self.block_header.serialize());
        fields.push(self.prover_address.to_field());
        fields.storage
    }
}
