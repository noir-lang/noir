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
        previous_kernel_data::PreviousKernelData,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        private_kernel::private_call_data::PrivateCallData,
        membership_witness::{
            ContractLeafMembershipWitness,
            FunctionLeafMembershipWitness,
            ReadRequestMembershipWitness,
        },
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
    tests::fixtures,
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
        RETURN_VALUES_LENGTH,
        VK_TREE_HEIGHT,
    },
};

struct PrivateAppInputs {
    call_context: CallContext, 
    historical_block_data: HistoricalBlockData,
    contract_deployment_data: ContractDeploymentData,
    args_hash: Field,
    chain_id: Field,
    version: Field,
}

struct PrivateCircuitPublicInputsBuilder {
    call_context: CallContext,
    
    args_hash: Field,
    return_values: BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<Field, MAX_READ_REQUESTS_PER_CALL>,
    pending_read_requests: BoundedVec<Field, MAX_PENDING_READ_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,
    nullified_commitments: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack: BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack: BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    encrypted_logs_hash : [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash : [Field; NUM_FIELDS_PER_SHA256],

    encrypted_log_preimages_length : Field,
    unencrypted_log_preimages_length : Field,

    historical_block_data : HistoricalBlockData,

    contract_deployment_data : ContractDeploymentData,

    chain_id : Field,
    version : Field,
}

impl PrivateCircuitPublicInputsBuilder {
    pub fn new(inputs: PrivateAppInputs) -> Self {
        Self {
            call_context: inputs.call_context,

            args_hash: inputs.args_hash,
            return_values : BoundedVec::new(0),

            read_requests : BoundedVec::new(0),
            pending_read_requests : BoundedVec::new(0),

            new_commitments : BoundedVec::new(0),
            new_nullifiers : BoundedVec::new(0),
            nullified_commitments : BoundedVec::new(0),

            private_call_stack : BoundedVec::new(0),
            public_call_stack : BoundedVec::new(0),
            new_l2_to_l1_msgs : BoundedVec::new(0),

            encrypted_logs_hash : [0; NUM_FIELDS_PER_SHA256],
            unencrypted_logs_hash : [0; NUM_FIELDS_PER_SHA256],

            encrypted_log_preimages_length : 0,
            unencrypted_log_preimages_length : 0,

            historical_block_data : inputs.historical_block_data,

            contract_deployment_data : inputs.contract_deployment_data,

            chain_id : inputs.chain_id,
            version : inputs.version,
        }
    }

    pub fn finish(self) ->  PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context : self.call_context,
            args_hash : self.args_hash,
            return_values : self.return_values.storage,

            read_requests : self.read_requests.storage,
            pending_read_requests : self.pending_read_requests.storage,

            new_commitments : self.new_commitments.storage,
            new_nullifiers : self.new_nullifiers.storage,
            nullified_commitments : self.nullified_commitments.storage,

            private_call_stack : self.private_call_stack.storage,
            public_call_stack : self.public_call_stack.storage,
            new_l2_to_l1_msgs : self.new_l2_to_l1_msgs.storage,

            encrypted_logs_hash : self.encrypted_logs_hash,
            unencrypted_logs_hash : self.unencrypted_logs_hash,

            encrypted_log_preimages_length : self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length : self.unencrypted_log_preimages_length,

            historical_block_data : self.historical_block_data,

            contract_deployment_data : self.contract_deployment_data,

            chain_id : self.chain_id,
            version : self.version,
        }
    }
}

fn build_contract_deployment_data(is_constructor: bool) -> ContractDeploymentData {
    let mut contract_deployment_data: ContractDeploymentData = dep::std::unsafe::zeroed();
    if is_constructor {
        let contract_data = fixtures::contracts::default_contract;
        let constructor = fixtures::contract_functions::default_constructor;
        contract_deployment_data = ContractDeploymentData {
            deployer_public_key: fixtures::DEPLOYER_PUBLIC_KEY,
            constructor_vk_hash: constructor.vk_hash,
            function_tree_root: contract_data.function_tree_root,
            contract_address_salt: contract_data.contract_address_salt,
            portal_contract_address: contract_data.portal_contract_address,
        };
    }
    contract_deployment_data
}

fn build_tx_context(is_constructor: bool) -> TxContext {
    let contract_deployment_data = build_contract_deployment_data(is_constructor);
     TxContext {
        is_fee_payment_tx: false,
        is_rebate_payment_tx: false,
        is_contract_deployment_tx: is_constructor,
        contract_deployment_data,
        chain_id: 1,
        version: 0,
    }
}

struct PreviousKernelDataBuilder {
    end: CombinedAccumulatedDataBuilder,
    block_data: HistoricalBlockData,
    tx_context: TxContext,
    is_private: bool,
    proof: Proof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl PreviousKernelDataBuilder {
    pub fn new(is_constructor: bool) -> Self {
        let mut end: CombinedAccumulatedDataBuilder = dep::std::unsafe::zeroed();
        end.new_nullifiers.push(321); // 0th nullifier must be non-zero.
        end.nullified_commitments.push(EMPTY_NULLIFIED_COMMITMENT);
        end.private_call_stack.push(1);

        let tx_context = build_tx_context(is_constructor);

        PreviousKernelDataBuilder {
            end,
            block_data: fixtures::HISTORICAL_BLOCK_DATA,
            tx_context,
            is_private: true,
            proof: Proof {},
            vk: VerificationKey {},
            vk_index: 0,
            vk_path: [0; VK_TREE_HEIGHT],
        }
    }

    pub fn add_read_request_for_transient_commitment(&mut self, commitment_index: Field) -> Field {
        let new_read_request_index = self.end.read_requests.len();
        self.end.read_requests.push(self.end.new_commitments.get(commitment_index));
        new_read_request_index
    }

    pub fn append_new_commitments(&mut self, num_new_commitments: Field) {
        let mocked_value_offset = self.end.new_commitments.len() + 1;
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            if i as u64 < num_new_commitments as u64 {
                // The default value is its index + 1.
                self.end.new_commitments.push(i + mocked_value_offset);
            }
        }
    }

    pub fn append_new_nullifiers(&mut self, num_extra_nullifier: Field) {
        let first_nullifier = self.end.new_nullifiers.get(0);
        let mocked_value_offset = first_nullifier + self.end.new_nullifiers.len();
        for i in 1..MAX_NEW_NULLIFIERS_PER_TX {
            if i as u64 <= num_extra_nullifier as u64 {
                // The default value is its index + the value of the first nullifier.
                self.end.new_nullifiers.push(i + mocked_value_offset);
            }
        }
    }

    pub fn set_encrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.end.encrypted_logs_hash = hash;
        self.end.encrypted_log_preimages_length = preimages_length;
    }

    pub fn set_unencrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.end.unencrypted_logs_hash = hash;
        self.end.unencrypted_log_preimages_length = preimages_length;
    }

    pub fn finish(self) -> PreviousKernelData {
        let public_inputs = KernelCircuitPublicInputs {
            end: self.end.finish(),
            constants: CombinedConstantData {
                block_data: self.block_data,
                tx_context: self.tx_context,
            },
            is_private: self.is_private,
        };
        PreviousKernelData {
            public_inputs,
            proof: self.proof,
            vk: self.vk,
            vk_index: self.vk_index,
            vk_path: self.vk_path,
        }
    }
}

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
