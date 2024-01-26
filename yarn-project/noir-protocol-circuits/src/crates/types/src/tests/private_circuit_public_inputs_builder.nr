use crate::{
    abis::{
        call_context::CallContext,
        complete_address::CompleteAddress,
        block_header::BlockHeader,
        nullifier_key_validation_request::NullifierKeyValidationRequest,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    },
    contrakt::deployment_data::ContractDeploymentData,
    hash::{compute_constructor_hash, hash_args},
    tests::{
        fixtures,
        testing_harness::build_contract_deployment_data,
    },
    utils::bounded_vec::BoundedVec,
};
use crate::constants::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        NUM_FIELDS_PER_SHA256,
        RETURN_VALUES_LENGTH,
};

struct PrivateCircuitPublicInputsBuilder {
    call_context: CallContext,
    
    args_hash: Field,
    return_values: BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<SideEffect, MAX_READ_REQUESTS_PER_CALL>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<SideEffect, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack_hashes: BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    block_header: BlockHeader,

    contract_deployment_data: ContractDeploymentData,

    chain_id: Field,
    version: Field,
}

impl PrivateCircuitPublicInputsBuilder {
    pub fn new(is_constructor: bool) -> Self {
        let mut public_inputs: PrivateCircuitPublicInputsBuilder = dep::std::unsafe::zeroed();

        let args_hash = hash_args([]);

        let contract_data = fixtures::contracts::default_contract;
        let portal_contract_address = contract_data.portal_contract_address;

        let contract_function = if is_constructor {
            fixtures::contract_functions::default_constructor
        } else {
            fixtures::contract_functions::default_private_function
        };
        let function_data = contract_function.data;

        let contract_deployment_data = build_contract_deployment_data(is_constructor);

        let contract_address = if is_constructor {
            let constructor = fixtures::contract_functions::default_constructor;
            let constructor_hash = compute_constructor_hash(constructor.data, args_hash, constructor.vk_hash);
            CompleteAddress::compute(
                contract_deployment_data.deployer_public_key,
                contract_deployment_data.contract_address_salt,
                contract_deployment_data.function_tree_root,
                constructor_hash,
            ).address
        } else {
            contract_data.address
        };

        let call_context = CallContext {
            msg_sender: fixtures::contracts::parent_contract.address,
            storage_contract_address: contract_address,
            portal_contract_address,
            function_selector: function_data.selector,
            is_delegate_call: false,
            is_static_call: false,
            is_contract_deployment: is_constructor,
            start_side_effect_counter: 0,
        };
        public_inputs.call_context = call_context;
        public_inputs.args_hash = args_hash;
        public_inputs.contract_deployment_data = contract_deployment_data;
        public_inputs.block_header = fixtures::BLOCK_HEADER;
        public_inputs.chain_id = 0;
        public_inputs.version = 1;

        public_inputs
    }

    pub fn finish(self) ->  PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.call_context,
            args_hash: self.args_hash,
            return_values: self.return_values.storage,

            read_requests: self.read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,

            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,

            private_call_stack_hashes: self.private_call_stack_hashes.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            end_side_effect_counter: 10,

            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,

            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,

            block_header: self.block_header,

            contract_deployment_data: self.contract_deployment_data,

            chain_id: self.chain_id,
            version: self.version,
        }
    }
}
