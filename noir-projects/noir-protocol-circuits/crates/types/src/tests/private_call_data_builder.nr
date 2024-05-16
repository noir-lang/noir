use crate::{
    abis::{
    gas_settings::GasSettings, call_request::{CallerContext, CallRequest},
    private_call_stack_item::PrivateCallStackItem, function_data::FunctionData,
    max_block_number::MaxBlockNumber, private_circuit_public_inputs::PrivateCircuitPublicInputs,
    private_kernel::private_call_data::PrivateCallData, log_hash::LogHash
},
    merkle_tree::membership::MembershipWitness,
    address::{AztecAddress, EthAddress, SaltedInitializationHash, PublicKeysHash},
    recursion::{proof::RecursiveProof, verification_key::VerificationKey},
    tests::{fixtures, private_circuit_public_inputs_builder::PrivateCircuitPublicInputsBuilder},
    transaction::{tx_request::TxRequest, tx_context::TxContext}
};
use crate::constants::{MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, FUNCTION_TREE_HEIGHT};

struct PrivateCallDataBuilder {
    // Values of PrivateCallStackItem.
    contract_address: AztecAddress,
    public_inputs: PrivateCircuitPublicInputsBuilder,
    is_execution_request: bool,
    function_data: FunctionData,
    // The rest of the values of PrivateCallData.
    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    public_teardown_call_request: CallRequest,
    proof: RecursiveProof,
    vk: VerificationKey,
    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
    acir_hash: Field,
    gas_settings: GasSettings,
}

impl PrivateCallDataBuilder {
    pub fn new() -> Self {
        PrivateCallDataBuilder::new_with_counter(0)
    }

    pub fn new_with_counter(counter: u32) -> Self {
        let public_inputs = PrivateCircuitPublicInputsBuilder::new_with_counter(counter);

        let contract_data = fixtures::contracts::default_contract;
        let contract_function = fixtures::contract_functions::default_private_function;
        let function_data = contract_function.data;

        PrivateCallDataBuilder {
            contract_address: public_inputs.call_context.storage_contract_address,
            public_inputs,
            is_execution_request: false,
            function_data,
            private_call_stack: BoundedVec::new(),
            public_call_stack: BoundedVec::new(),
            public_teardown_call_request: CallRequest::empty(),
            proof: RecursiveProof::empty(),
            vk: VerificationKey::empty(),
            function_leaf_membership_witness: contract_function.membership_witness,
            salted_initialization_hash: contract_data.salted_initialization_hash,
            public_keys_hash: contract_data.public_keys_hash,
            contract_class_artifact_hash: contract_data.artifact_hash,
            contract_class_public_bytecode_commitment: contract_data.public_bytecode_commitment,
            acir_hash: contract_function.acir_hash,
            gas_settings: public_inputs.gas_settings
        }
    }

    pub fn is_delegate_call(&mut self) -> Self {
        let _ = self.public_inputs.is_delegate_call();
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        let _ = self.public_inputs.is_static_call();
        *self
    }

    pub fn build_tx_request(self) -> TxRequest {
        let tx_context = self.public_inputs.build_tx_context();
        TxRequest {
            origin: self.contract_address,
            args_hash: self.public_inputs.args_hash,
            tx_context,
            function_data: self.function_data
        }
    }

    pub fn build_call_request(self) -> CallRequest {
        let hash = self.build_call_stack_item().hash();
        let is_delegate_call = self.public_inputs.call_context.is_delegate_call;
        let caller_context = if is_delegate_call {
            CallerContext {
                msg_sender: fixtures::MSG_SENDER,
                storage_contract_address: self.public_inputs.call_context.storage_contract_address
            }
        } else {
            CallerContext::empty()
        };
        CallRequest {
            hash,
            caller_contract_address: self.public_inputs.call_context.msg_sender,
            caller_context,
            start_side_effect_counter: self.public_inputs.counter_start,
            end_side_effect_counter: self.public_inputs.counter_end
        }
    }

    pub fn append_private_call_requests(&mut self, num_requests: u64, is_delegate_call: bool) {
        let hash_offset = 7070 + self.private_call_stack.len();
        for i in 0..self.private_call_stack.max_len() {
            if i < num_requests {
                let hash = (hash_offset + i) as Field;
                let request = self.public_inputs.generate_call_request(hash, is_delegate_call);
                self.private_call_stack.push(request);
                self.public_inputs.add_private_call_request(hash);
            }
        }
    }

    pub fn append_public_call_requests(&mut self, num_requests: u64, is_delegate_call: bool) {
        let hash_offset = 7070 + self.public_call_stack.len() as Field;
        for i in 0..self.public_call_stack.max_len() {
            if i < num_requests {
                let hash = hash_offset + i as Field;
                let request = self.public_inputs.generate_call_request(hash, is_delegate_call);
                self.public_call_stack.push(request);
                self.public_inputs.add_public_call_request(hash);
            }
        }
    }

    pub fn add_teardown_call_request(&mut self, is_delegate_call: bool) {
        let hash = 909090;
        self.public_teardown_call_request = self.public_inputs.generate_call_request(hash, is_delegate_call);
        self.public_inputs.add_teardown_call_request(hash);
    }

    fn build_call_stack_item(self) -> PrivateCallStackItem {
        PrivateCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            public_inputs: self.public_inputs.finish()
        }
    }

    pub fn get_call_stack_item_hash(self) -> Field {
        self.build_call_stack_item().hash()
    }

    pub fn finish(self) -> PrivateCallData {
        PrivateCallData {
            call_stack_item: self.build_call_stack_item(),
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            public_teardown_call_request: self.public_teardown_call_request,
            proof: self.proof,
            vk: self.vk,
            function_leaf_membership_witness: self.function_leaf_membership_witness,
            salted_initialization_hash: self.salted_initialization_hash,
            public_keys_hash: self.public_keys_hash,
            contract_class_artifact_hash: self.contract_class_artifact_hash,
            contract_class_public_bytecode_commitment: self.contract_class_public_bytecode_commitment,
            acir_hash: self.acir_hash
        }
    }
}
