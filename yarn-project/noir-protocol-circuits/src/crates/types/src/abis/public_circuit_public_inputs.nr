use crate::contrakt::storage_read::StorageRead;
use crate::contrakt::storage_update_request::StorageUpdateRequest;
use crate::address::Address;
use crate::abis::historical_block_data::HistoricalBlockData;
use dep::aztec::constants_gen;
use crate::hash::{NUM_FIELDS_PER_SHA256};
use dep::aztec::constants_gen::{
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    RETURN_VALUES_LENGTH,
};
use crate::abis::call_context::CallContext;
use crate::utils::bounded_vec::BoundedVec;

struct PublicCircuitPublicInputs{
    call_context : CallContext,

    args_hash : Field,
    return_values : [Field; RETURN_VALUES_LENGTH],

    contract_storage_update_requests : [StorageUpdateRequest;MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
    contract_storage_reads : [StorageRead; MAX_PUBLIC_DATA_READS_PER_CALL],

    public_call_stack : [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_commitments : [Field; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers : [Field; MAX_NEW_NULLIFIERS_PER_CALL],
    new_l2_to_l1_msgs : [Field; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],

    unencrypted_logs_hash : [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    unencrypted_log_preimages_length : Field,

    historical_block_data : HistoricalBlockData,

    prover_address : Address,
}


impl PublicCircuitPublicInputs{
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3059) : Reuse aztec-nr
    fn hash(self) -> Field {        
        let mut inputs: BoundedVec<Field, constants_gen::PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH> = BoundedVec::new(0);
        inputs.push(self.call_context.hash());
        inputs.push(self.args_hash);
        inputs.push_array(self.return_values);
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            inputs.push(self.contract_storage_update_requests[i].hash());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            inputs.push(self.contract_storage_reads[i].hash());
        }
        inputs.push_array(self.public_call_stack);
        inputs.push_array(self.new_commitments);
        inputs.push_array(self.new_nullifiers);
        inputs.push_array(self.new_l2_to_l1_msgs);
        inputs.push_array(self.unencrypted_logs_hash);
        inputs.push(self.unencrypted_log_preimages_length);
        inputs.push_array(self.historical_block_data.to_array());
        inputs.push(self.prover_address.to_field());

        assert_eq(inputs.len(), constants_gen::PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH, "Incorrect number of input fields when hashing PublicCircuitPublicInputs");

        dep::std::hash::pedersen_hash_with_separator(inputs.storage, constants_gen::GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS)
    }
}
