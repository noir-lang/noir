use crate::{
    abis::{
    call_request::{CallerContext, CallRequest}, private_call_stack_item::PrivateCallStackItem,
    function_data::FunctionData,
    membership_witness::{ContractLeafMembershipWitness, FunctionLeafMembershipWitness, ReadRequestMembershipWitness},
    private_circuit_public_inputs::{PrivateCircuitPublicInputs},
    private_kernel::private_call_data::PrivateCallData
},
    address::{AztecAddress, EthAddress, SaltedInitializationHash, PublicKeysHash},
    mocked::{Proof, VerificationKey},
    tests::{
    fixtures, private_circuit_public_inputs_builder::PrivateCircuitPublicInputsBuilder,
    testing_harness::build_tx_context
},
    transaction::{tx_request::TxRequest}
};
use crate::constants::{
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_READ_REQUESTS_PER_CALL, NUM_FIELDS_PER_SHA256
};

struct PrivateCallDataBuilder {
    // Values of PrivateCallStackItem.
    contract_address: AztecAddress,
    public_inputs: PrivateCircuitPublicInputsBuilder,
    is_execution_request: bool,
    function_data: FunctionData,
    // The rest of the values of PrivateCallData.
    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    proof: Proof,
    vk: VerificationKey,
    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: FunctionLeafMembershipWitness,
    read_request_membership_witnesses: BoundedVec<ReadRequestMembershipWitness, MAX_READ_REQUESTS_PER_CALL>,
    portal_contract_address: EthAddress,
    acir_hash: Field,
}

impl PrivateCallDataBuilder {
    pub fn new(is_constructor: bool) -> Self {
        let public_inputs = PrivateCircuitPublicInputsBuilder::new(is_constructor);

        let contract_data = fixtures::contracts::default_contract;
        let contract_function = if is_constructor {
            fixtures::contract_functions::default_constructor
        } else {
            fixtures::contract_functions::default_private_function
        };
        let function_data = contract_function.data;

        PrivateCallDataBuilder {
            contract_address: public_inputs.call_context.storage_contract_address,
            public_inputs,
            is_execution_request: false,
            function_data,
            private_call_stack: dep::std::unsafe::zeroed(),
            public_call_stack: dep::std::unsafe::zeroed(),
            proof: Proof {},
            vk: VerificationKey {},
            function_leaf_membership_witness: contract_function.membership_witness,
            salted_initialization_hash: contract_data.salted_initialization_hash,
            public_keys_hash: contract_data.public_keys_hash,
            contract_class_artifact_hash: contract_data.artifact_hash,
            contract_class_public_bytecode_commitment: contract_data.public_bytecode_commitment,
            read_request_membership_witnesses: dep::std::unsafe::zeroed(),
            portal_contract_address: public_inputs.call_context.portal_contract_address,
            acir_hash: contract_function.acir_hash
        }
    }

    pub fn is_delegate_call(&mut self) -> Self {
        self.public_inputs.call_context.is_delegate_call = true;
        self.public_inputs.call_context.storage_contract_address = fixtures::contracts::parent_contract.address;
        self.public_inputs.call_context.portal_contract_address = fixtures::contracts::parent_contract.portal_contract_address;
        self.public_inputs.call_context.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        self.public_inputs.call_context.is_static_call = true;
        *self
    }

    pub fn is_internal_call(&mut self) -> Self {
        self.function_data.is_internal = true;
        self.public_inputs.call_context.msg_sender = self.contract_address;
        let contract_function = fixtures::contract_functions::default_internal_private_function;
        self.function_data = contract_function.data;
        self.function_leaf_membership_witness = contract_function.membership_witness;
        self.acir_hash = contract_function.acir_hash;
        *self
    }

    pub fn build_tx_request(self) -> TxRequest {
        let tx_context = build_tx_context(
            self.public_inputs.call_context.is_contract_deployment,
            self.public_inputs.args_hash
        );
        TxRequest {
            origin: self.contract_address,
            args_hash: self.public_inputs.args_hash,
            tx_context,
            function_data: self.function_data
        }
    }

    pub fn append_private_call_requests(&mut self, num_requests: u64, is_delegate_call: bool) {
        let (hashes, call_requests) = self.generate_call_requests(self.private_call_stack, num_requests, is_delegate_call);
        self.public_inputs.private_call_stack_hashes.extend_from_bounded_vec(hashes);
        self.private_call_stack.extend_from_bounded_vec(call_requests);
    }

    pub fn append_public_call_requests(&mut self, num_requests: u64, is_delegate_call: bool) {
        let (hashes, call_requests) = self.generate_call_requests(self.public_call_stack, num_requests, is_delegate_call);
        self.public_inputs.public_call_stack_hashes.extend_from_bounded_vec(hashes);
        self.public_call_stack.extend_from_bounded_vec(call_requests);
    }

    fn generate_call_requests<N>(
        self,
        requests: BoundedVec<CallRequest, N>,
        num_requests: u64,
        is_delegate_call: bool
    ) -> (BoundedVec<Field, N>, BoundedVec<CallRequest, N>) {
        let value_offset = requests.len();
        let mut caller_context = CallerContext::empty();
        if is_delegate_call {
            let call_context = self.public_inputs.call_context;
            caller_context.msg_sender = call_context.msg_sender;
            caller_context.storage_contract_address = call_context.storage_contract_address;
        }
        let mut call_requests: BoundedVec<CallRequest, N> = BoundedVec::new();
        let mut hashes: BoundedVec<Field, N> = BoundedVec::new();
        let mut exceeded_len = false;
        for i in 0..N {
            exceeded_len |= i == num_requests;
            if !exceeded_len {
                // The default hash is its index + 7788.
                let hash = (value_offset + 7788) as Field;
                let request = CallRequest {
                    hash,
                    caller_contract_address: self.contract_address,
                    caller_context,
                    // TODO: populate these
                    start_side_effect_counter: 0,
                    end_side_effect_counter: 0
                };
                hashes.push(hash);
                call_requests.push(request);
            }
        }
        (hashes, call_requests)
    }

    pub fn append_read_requests(&mut self, num_read_requests: u64) {
        let (read_requests, read_request_membership_witnesses) = fixtures::read_requests::generate_read_requests(num_read_requests);
        self.public_inputs.read_requests.extend_from_bounded_vec(read_requests);
        self.read_request_membership_witnesses.extend_from_bounded_vec(read_request_membership_witnesses);
    }

    pub fn append_transient_read_requests(&mut self, num_read_requests: u64) {
        let (read_requests, read_request_membership_witnesses) = fixtures::read_requests::generate_transient_read_requests(num_read_requests);
        self.public_inputs.read_requests.extend_from_bounded_vec(read_requests);
        self.read_request_membership_witnesses.extend_from_bounded_vec(read_request_membership_witnesses);
    }

    pub fn set_encrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.public_inputs.encrypted_logs_hash = hash;
        self.public_inputs.encrypted_log_preimages_length = preimages_length;
    }

    pub fn set_unencrypted_logs(&mut self, hash: [Field; NUM_FIELDS_PER_SHA256], preimages_length: Field) {
        self.public_inputs.unencrypted_logs_hash = hash;
        self.public_inputs.unencrypted_log_preimages_length = preimages_length;
    }

    pub fn get_call_stack_item_hash(self) -> Field {
        let call_stack_item = PrivateCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            public_inputs: self.public_inputs.finish()
        };
        call_stack_item.hash()
    }

    pub fn finish(self) -> PrivateCallData {
        let call_stack_item = PrivateCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            public_inputs: self.public_inputs.finish()
        };

        PrivateCallData {
            call_stack_item,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            proof: self.proof,
            vk: self.vk,
            function_leaf_membership_witness: self.function_leaf_membership_witness,
            salted_initialization_hash: self.salted_initialization_hash,
            public_keys_hash: self.public_keys_hash,
            contract_class_artifact_hash: self.contract_class_artifact_hash,
            contract_class_public_bytecode_commitment: self.contract_class_public_bytecode_commitment,
            read_request_membership_witnesses: self.read_request_membership_witnesses.storage,
            portal_contract_address: self.portal_contract_address,
            acir_hash: self.acir_hash
        }
    }
}
