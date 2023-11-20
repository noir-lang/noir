use crate::{
    abis::{
        call_context::CallContext,
        call_stack_item::PrivateCallStackItem,
        combined_constant_data::CombinedConstantData,
        combined_accumulated_data::CombinedAccumulatedDataBuilder,
        complete_address::CompleteAddress,
        function_data::FunctionData,
        historical_block_data::HistoricalBlockData,
        kernel_circuit_public_inputs::KernelCircuitPublicInputs,
        membership_witness::{
            ContractLeafMembershipWitness,
            FunctionLeafMembershipWitness,
            ReadRequestMembershipWitness,
        },
        previous_kernel_data::PreviousKernelData,
        private_circuit_public_inputs::{PrivateCircuitPublicInputs, PrivateCircuitPublicInputsBuilder},
        private_kernel::private_call_data::PrivateCallData,
        public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest,
    },
    address::{Address, EthAddress},
    contrakt::{
        deployment_data::ContractDeploymentData,
    },
    hash::{
        NUM_FIELDS_PER_SHA256,
        compute_constructor_hash,
    },
    mocked::{Proof, VerificationKey},
    point::Point,
    tests::{
        fixtures,
        testing_harness::{build_contract_deployment_data, build_tx_context},
    },
    transaction::{
        request::TxRequest,
        context::TxContext,
    },
    utils::{
        bounded_vec::BoundedVec,
    },
};
use dep::aztec::{
    abi::hash_args,
    constants_gen::{
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_TX,
        MAX_PENDING_READ_REQUESTS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_READ_REQUESTS_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
        RETURN_VALUES_LENGTH,
        VK_TREE_HEIGHT,
    },
};

struct PrivateCallDataBuilder {
    // Values of PrivateCallStackItem.
    contract_address: Address,
    public_inputs: PrivateCircuitPublicInputsBuilder,
    is_execution_request: bool,
    function_data: FunctionData,
    // The rest of the values of PrivateCallData.
    private_call_stack_preimages: [PrivateCallStackItem; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    proof: Proof,
    vk: VerificationKey,
    function_leaf_membership_witness: FunctionLeafMembershipWitness,
    contract_leaf_membership_witness: ContractLeafMembershipWitness,
    read_request_membership_witnesses: BoundedVec<ReadRequestMembershipWitness, MAX_READ_REQUESTS_PER_CALL>,
    portal_contract_address: EthAddress,
    acir_hash: Field,
}

impl PrivateCallDataBuilder {
    pub fn new(is_constructor: bool) -> Self {
        let args_hash = hash_args([]);

        let contract_data = fixtures::contracts::default_contract;
        let portal_contract_address = contract_data.portal_contract_address;

        let contract_function = if is_constructor {
            fixtures::contract_functions::default_constructor
        } else {
            fixtures::contract_functions::default_contract_function
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
            msg_sender: fixtures::MSG_SENDER,
            storage_contract_address: contract_address,
            portal_contract_address,
            function_selector: function_data.selector,
            is_delegate_call: false,
            is_static_call: false,
            is_contract_deployment: is_constructor,
        };

        let mut public_inputs: PrivateCircuitPublicInputsBuilder = dep::std::unsafe::zeroed();
        public_inputs.call_context = call_context;
        public_inputs.args_hash = args_hash;
        public_inputs.historical_block_data = fixtures::HISTORICAL_BLOCK_DATA;
        public_inputs.contract_deployment_data = contract_deployment_data;
        public_inputs.chain_id = 0;
        public_inputs.version = 1;

        PrivateCallDataBuilder {
            contract_address,
            public_inputs,
            is_execution_request: false,
            function_data,
            private_call_stack_preimages: dep::std::unsafe::zeroed(),
            proof: Proof {},
            vk: VerificationKey {},
            function_leaf_membership_witness: contract_function.membership_witness,
            contract_leaf_membership_witness: contract_data.membership_witness,
            read_request_membership_witnesses: dep::std::unsafe::zeroed(),
            portal_contract_address,
            acir_hash: contract_function.acir_hash,
        }
    }

    pub fn build_tx_request(self) -> TxRequest {
        let tx_context = build_tx_context(self.public_inputs.call_context.is_contract_deployment);
        TxRequest {
            origin: self.contract_address,
            args_hash: self.public_inputs.args_hash,
            tx_context,
            function_data: self.function_data,
        }
    }

    pub fn append_read_requests(&mut self, num_read_requests: Field) {
        let (read_requests, read_request_membership_witnesses) = fixtures::read_requests::generate_read_requests(num_read_requests);
        self.public_inputs.read_requests.push_vec(read_requests);
        self.read_request_membership_witnesses.push_vec(read_request_membership_witnesses);
    }

    pub fn append_transient_read_requests(&mut self, num_read_requests: Field) {
        let (read_requests, read_request_membership_witnesses) = fixtures::read_requests::generate_transient_read_requests(num_read_requests);
        self.public_inputs.read_requests.push_vec(read_requests);
        self.read_request_membership_witnesses.push_vec(read_request_membership_witnesses);
    }

    pub fn set_encrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.public_inputs.encrypted_logs_hash = hash;
        self.public_inputs.encrypted_log_preimages_length = preimages_length;
    }

    pub fn set_unencrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.public_inputs.unencrypted_logs_hash = hash;
        self.public_inputs.unencrypted_log_preimages_length = preimages_length;
    }

    pub fn finish(self) -> PrivateCallData {
        let call_stack_item = PrivateCallStackItem {
            contract_address: self.contract_address,
            public_inputs: self.public_inputs.finish(),
            is_execution_request: false,
            function_data: self.function_data,
        };

        PrivateCallData {
            call_stack_item,
            private_call_stack_preimages: self.private_call_stack_preimages,
            proof: self.proof,
            vk: self.vk,
            function_leaf_membership_witness: self.function_leaf_membership_witness,
            contract_leaf_membership_witness: self.contract_leaf_membership_witness,
            read_request_membership_witnesses: self.read_request_membership_witnesses.storage,
            portal_contract_address: self.portal_contract_address,
            acir_hash: self.acir_hash,
        }
    }
}
