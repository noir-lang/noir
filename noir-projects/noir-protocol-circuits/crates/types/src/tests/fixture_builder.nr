use crate::{
    abis::{
    gas::Gas, gas_settings::GasSettings, call_context::CallContext,
    accumulated_data::{
    CombinedAccumulatedData, PrivateAccumulatedData, PrivateAccumulatedDataBuilder,
    PublicAccumulatedData, PublicAccumulatedDataBuilder
},
    function_data::FunctionData, global_variables::GlobalVariables,
    combined_constant_data::CombinedConstantData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputs, PublicKernelCircuitPublicInputs},
    kernel_data::KernelData, public_kernel_data::PublicKernelData, max_block_number::MaxBlockNumber,
    private_kernel_data::PrivateKernelData, note_hash::{NoteHash, ScopedNoteHash},
    nullifier::{Nullifier, ScopedNullifier}, private_call_request::PrivateCallRequest,
    private_call_stack_item::PrivateCallStackItem,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    private_kernel::private_call_data::PrivateCallData, public_call_request::PublicCallRequest,
    public_call_stack_item_compressed::PublicCallStackItemCompressed, public_data_read::PublicDataRead,
    public_data_update_request::PublicDataUpdateRequest, read_request::{ReadRequest, ScopedReadRequest},
    log_hash::{LogHash, NoteLogHash, ScopedLogHash, EncryptedLogHash, ScopedEncryptedLogHash},
    validation_requests::{
    ValidationRequests, ValidationRequestsBuilder, KeyValidationRequest,
    KeyValidationRequestAndGenerator, ScopedKeyValidationRequestAndGenerator
}
},
    address::{AztecAddress, EthAddress, SaltedInitializationHash, PublicKeysHash},
    constants::{
    FUNCTION_TREE_HEIGHT, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_FIELD_VALUE,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX, VK_TREE_HEIGHT,
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
    PRIVATE_CALL_REQUEST_LENGTH, PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH
},
    hash::{
    compute_l2_to_l1_hash, compute_tx_logs_hash, compute_tx_note_logs_hash, compute_siloed_nullifier,
    silo_encrypted_log_hash, silo_note_hash, silo_unencrypted_log_hash
},
    header::Header, merkle_tree::membership::MembershipWitness,
    messaging::l2_to_l1_message::{L2ToL1Message, ScopedL2ToL1Message},
    partial_state_reference::PartialStateReference, tests::fixtures,
    transaction::{tx_context::TxContext, tx_request::TxRequest}, traits::Empty,
    recursion::{verification_key::VerificationKey, proof::{NestedRecursiveProof, RecursiveProof}},
    point::Point
};

fn subarray<T, let N: u32, let M: u32>(arr: [T; N]) -> [T; M] {
    assert(N >= M, "cannot call subarray on a smaller array");
    let mut new_arr = [arr[0]; M];
    for i in 0..M {
        new_arr[i] = arr[i];
    }
    new_arr
}

fn vec_reverse<T, let N: u32>(vec: BoundedVec<T, N>) -> BoundedVec<T, N> {
    let mut reversed = BoundedVec::new();
    let len = vec.len();
    for i in 0..N {
        if i < len {
            reversed.push(vec.storage[len - i - 1]);
        }
    }
    reversed
}

struct FixtureBuilder {
    contract_address: AztecAddress,
    storage_contract_address: AztecAddress,
    msg_sender: AztecAddress,
    is_delegate_call: bool,
    is_static_call: bool,

    // Fees.
    is_fee_payer: bool,
    fee_payer: AztecAddress,
    public_teardown_call_stack: BoundedVec<PublicCallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,

    // Constant data.
    historical_header: Header,
    tx_context: TxContext,
    global_variables: GlobalVariables,

    // Accumulated data.
    note_hashes: BoundedVec<ScopedNoteHash, MAX_NOTE_HASHES_PER_TX>,
    nullifiers: BoundedVec<ScopedNullifier, MAX_NULLIFIERS_PER_TX>,
    l2_to_l1_msgs: BoundedVec<ScopedL2ToL1Message, MAX_L2_TO_L1_MSGS_PER_TX>,
    note_encrypted_logs_hashes: BoundedVec<NoteLogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    encrypted_logs_hashes: BoundedVec<ScopedEncryptedLogHash, MAX_ENCRYPTED_LOGS_PER_TX>,
    unencrypted_logs_hashes: BoundedVec<ScopedLogHash, MAX_UNENCRYPTED_LOGS_PER_TX>,
    note_encrypted_logs_hash: Field,
    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,
    note_encrypted_log_preimages_length: Field,
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,
    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    private_call_requests: BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_requests: BoundedVec<PublicCallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    gas_used: Gas,
    non_revertible_gas_used: Gas,
    revert_code: u8,

    // Validation requests.
    max_block_number: MaxBlockNumber,
    note_hash_read_requests: BoundedVec<ScopedReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    nullifier_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifier_non_existent_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    scoped_key_validation_requests_and_generators: BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,

    // Function.
    function_data: FunctionData,
    args_hash: Field,
    returns_hash: Field,

    // Private call.
    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
    acir_hash: Field,

    // Proof.
    proof: NestedRecursiveProof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
    vk_tree_root: Field,

    // Counters.
    min_revertible_side_effect_counter: u32,
    counter_start: u32,
    counter: u32,

    // States.
    start_state: PartialStateReference,

    // Mock data.
    value_offset: Field,
}

impl FixtureBuilder {
    pub fn new() -> Self {
        FixtureBuilder::new_from_counter(0)
    }

    pub fn new_from_counter(counter_start: u32) -> Self {
        let mut builder = FixtureBuilder::empty();

        let contract_data = fixtures::contracts::default_contract;
        let contract_function = fixtures::contract_functions::default_private_function;
        let contract_address = contract_data.address;

        builder.tx_context = TxContext { chain_id: fixtures::CHAIN_ID, version: fixtures::VERSION, gas_settings: GasSettings::empty() };
        builder.contract_address = contract_address;
        builder.storage_contract_address = contract_address;
        builder.msg_sender = fixtures::contracts::parent_contract.address;
        builder.function_data = contract_function.data;
        builder.function_leaf_membership_witness = contract_function.membership_witness;
        builder.salted_initialization_hash = contract_data.salted_initialization_hash;
        builder.public_keys_hash = contract_data.public_keys_hash;
        builder.contract_class_artifact_hash = contract_data.artifact_hash;
        builder.contract_class_public_bytecode_commitment = contract_data.public_bytecode_commitment;
        builder.acir_hash = contract_function.acir_hash;
        builder.counter_start = counter_start;
        builder.counter = counter_start + 1;

        builder
    }

    pub fn as_parent_contract(&mut self) -> Self {
        self.contract_address = fixtures::contracts::parent_contract.address;
        self.storage_contract_address = fixtures::contracts::parent_contract.address;
        self.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn in_vk_tree(&mut self, vk_index: u32) -> Self {
        self.vk_index = vk_index;
        let vk_tree = fixtures::vk_tree::get_vk_merkle_tree();

        let vk_hash = vk_tree.leaves[vk_index];
        self.vk.hash = vk_hash;

        self.vk_path = vk_tree.get_sibling_path(vk_index);

        self.vk_tree_root = vk_tree.get_root();

        *self
    }

    pub fn is_delegate_call(&mut self) -> Self {
        self.is_delegate_call = true;
        self.storage_contract_address = fixtures::contracts::parent_contract.address;
        self.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        self.is_static_call = true;
        *self
    }

    pub fn is_first_call(&mut self) -> Self {
        self.msg_sender = AztecAddress::from_field(MAX_FIELD_VALUE);
        *self
    }

    pub fn to_constant_data(self) -> CombinedConstantData {
        CombinedConstantData {
            historical_header: self.historical_header,
            tx_context: self.tx_context,
            vk_tree_root: self.vk_tree_root,
            global_variables: self.global_variables
        }
    }

    pub fn build_tx_request(self) -> TxRequest {
        TxRequest {
            origin: self.contract_address,
            args_hash: self.args_hash,
            tx_context: self.tx_context,
            function_data: self.function_data
        }
    }

    pub fn build_call_context(self) -> CallContext {
        CallContext {
            msg_sender: self.msg_sender,
            storage_contract_address: self.storage_contract_address,
            function_selector: self.function_data.selector,
            is_delegate_call: self.is_delegate_call,
            is_static_call: self.is_static_call
        }
    }

    pub fn build_private_call_request(self) -> PrivateCallRequest {
        PrivateCallRequest {
            contract_address: self.contract_address,
            call_context: self.build_call_context(),
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            start_side_effect_counter: self.counter_start,
            end_side_effect_counter: self.counter
        }
    }

    pub fn to_private_circuit_public_inputs(self) -> PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.build_call_context(),
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            is_fee_payer: self.is_fee_payer,
            max_block_number: self.max_block_number,
            note_hash_read_requests: subarray(self.note_hash_read_requests.storage.map(|r: ScopedReadRequest| r.read_request)),
            nullifier_read_requests: subarray(self.nullifier_read_requests.storage.map(|r: ScopedReadRequest| r.read_request)),
            key_validation_requests_and_generators: subarray(
                self.scoped_key_validation_requests_and_generators.storage.map(|r: ScopedKeyValidationRequestAndGenerator| r.request)
            ),
            note_hashes: subarray(self.note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash)),
            nullifiers: subarray(self.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier)),
            private_call_requests: subarray(self.private_call_requests.storage),
            public_call_requests: subarray(self.public_call_requests.storage),
            public_teardown_call_request: self.public_teardown_call_stack.storage[0],
            l2_to_l1_msgs: subarray(self.l2_to_l1_msgs.storage.map(|r: ScopedL2ToL1Message| r.message)),
            start_side_effect_counter: self.counter_start,
            end_side_effect_counter: self.counter,
            note_encrypted_logs_hashes: subarray(self.note_encrypted_logs_hashes.storage),
            encrypted_logs_hashes: subarray(self.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.log_hash)),
            unencrypted_logs_hashes: subarray(self.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.log_hash)),
            historical_header: self.historical_header,
            tx_context: self.tx_context
        }
    }

    pub fn to_private_call_stack_item(self) -> PrivateCallStackItem {
        PrivateCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            public_inputs: self.to_private_circuit_public_inputs()
        }
    }

    pub fn to_private_call_data(self) -> PrivateCallData {
        PrivateCallData {
            call_stack_item: self.to_private_call_stack_item(),
            vk: self.vk,
            function_leaf_membership_witness: self.function_leaf_membership_witness,
            salted_initialization_hash: self.salted_initialization_hash,
            public_keys_hash: self.public_keys_hash,
            contract_class_artifact_hash: self.contract_class_artifact_hash,
            contract_class_public_bytecode_commitment: self.contract_class_public_bytecode_commitment,
            acir_hash: self.acir_hash
        }
    }

    pub fn to_private_accumulated_data_builder(self) -> PrivateAccumulatedDataBuilder {
        PrivateAccumulatedDataBuilder {
            note_hashes: self.note_hashes,
            nullifiers: self.nullifiers,
            l2_to_l1_msgs: self.l2_to_l1_msgs,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes,
            encrypted_logs_hashes: self.encrypted_logs_hashes,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes,
            public_call_requests: self.public_call_requests,
            private_call_stack: vec_reverse(self.private_call_requests)
        }
    }

    pub fn to_private_accumulated_data(self) -> PrivateAccumulatedData {
        self.to_private_accumulated_data_builder().finish()
    }

    pub fn to_public_accumulated_data_builder(self) -> PublicAccumulatedDataBuilder {
        let note_hashes = BoundedVec { storage: self.note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash), len: self.note_hashes.len() };
        let nullifiers = BoundedVec { storage: self.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier), len: self.nullifiers.len() };
        let l2_to_l1_msgs = BoundedVec {
            storage: self.l2_to_l1_msgs.storage.map(|m: ScopedL2ToL1Message| m.message.content),
            len: self.l2_to_l1_msgs.len()
        };
        let note_encrypted_logs_hashes = BoundedVec {
            storage: self.note_encrypted_logs_hashes.storage.map(|l: NoteLogHash| LogHash { value: l.value, counter: l.counter, length: l.length }),
            len: self.note_encrypted_logs_hashes.len()
        };
        let encrypted_logs_hashes = BoundedVec {
            storage: self.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.log_hash).map(|l: EncryptedLogHash| LogHash { value: l.value, counter: l.counter, length: l.length }),
            len: self.encrypted_logs_hashes.len()
        };

        PublicAccumulatedDataBuilder {
            note_hashes,
            nullifiers,
            l2_to_l1_msgs,
            note_encrypted_logs_hashes,
            encrypted_logs_hashes,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes,
            public_data_update_requests: self.public_data_update_requests,
            public_call_stack: self.public_call_requests,
            gas_used: self.gas_used
        }
    }

    pub fn to_public_accumulated_data(self) -> PublicAccumulatedData {
        self.to_public_accumulated_data_builder().finish()
    }

    pub fn to_exposed_public_accumulated_data(self) -> PublicAccumulatedData {
        PublicAccumulatedData {
            note_hashes: self.note_hashes.storage.map(|n: ScopedNoteHash| n.expose_to_public()),
            nullifiers: self.nullifiers.storage.map(|n: ScopedNullifier| n.expose_to_public()),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage.map(|m: ScopedL2ToL1Message| m.message.content),
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage.map(|l: NoteLogHash| l.expose_to_public()),
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.expose_to_public()),
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.expose_to_public()),
            public_data_update_requests: self.public_data_update_requests.storage,
            public_call_stack: self.public_call_requests.storage.map(|cr: PublicCallRequest| cr.expose_to_public()),
            gas_used: self.gas_used
        }
    }

    pub fn to_combined_accumulated_data(self) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            note_hashes: self.note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash.value),
            nullifiers: self.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier.value),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage.map(|m: ScopedL2ToL1Message| m.message.content),
            note_encrypted_logs_hash: self.note_encrypted_logs_hash,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.expose_to_public()),
            note_encrypted_log_preimages_length: self.note_encrypted_log_preimages_length,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            public_data_update_requests: self.public_data_update_requests.storage,
            gas_used: self.gas_used
        }
    }

    pub fn to_validation_requests(self) -> ValidationRequests {
        let validation_requests = ValidationRequestsBuilder {
            max_block_number: self.max_block_number,
            note_hash_read_requests: self.note_hash_read_requests,
            nullifier_read_requests: self.nullifier_read_requests,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests,
            scoped_key_validation_requests_and_generators: self.scoped_key_validation_requests_and_generators,
            public_data_reads: self.public_data_reads
        };
        validation_requests.finish()
    }

    pub fn to_private_kernel_circuit_public_inputs(self) -> PrivateKernelCircuitPublicInputs {
        let end = self.to_private_accumulated_data();
        let validation_requests = self.to_validation_requests();
        let constants = self.to_constant_data();
        let public_teardown_call_request = self.public_teardown_call_stack.get_unchecked(0);

        PrivateKernelCircuitPublicInputs {
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            end,
            validation_requests,
            constants,
            public_teardown_call_request,
            fee_payer: self.fee_payer
        }
    }

    pub fn to_private_kernel_data(self) -> PrivateKernelData {
        let public_inputs = self.to_private_kernel_circuit_public_inputs();
        PrivateKernelData { public_inputs, vk: self.vk, vk_index: self.vk_index, vk_path: self.vk_path }
    }

    pub fn to_public_kernel_circuit_public_inputs(self, revertible: bool) -> PublicKernelCircuitPublicInputs {
        // TODO: Split the data using self.min_revertible_side_effect_counter.
        let accumulated_data = self.to_public_accumulated_data();
        let end_non_revertible = if revertible {
            PublicAccumulatedData::empty()
        } else {
            accumulated_data
        };
        let end = if revertible {
            accumulated_data
        } else {
            PublicAccumulatedData::empty()
        };
        let validation_requests = self.to_validation_requests();
        let constants = self.to_constant_data();

        PublicKernelCircuitPublicInputs {
            end_non_revertible,
            end,
            validation_requests,
            constants,
            revert_code: self.revert_code,
            public_teardown_call_stack: self.public_teardown_call_stack.storage,
            fee_payer: self.fee_payer
        }
    }

    pub fn to_public_kernel_data(self, revertible: bool) -> PublicKernelData {
        let public_inputs = self.to_public_kernel_circuit_public_inputs(revertible);
        PublicKernelData { public_inputs, proof: self.proof, vk: self.vk, vk_index: self.vk_index, vk_path: self.vk_path }
    }

    pub fn to_kernel_circuit_public_inputs(self) -> KernelCircuitPublicInputs {
        let rollup_validation_requests = self.to_validation_requests().for_rollup;
        let end = self.to_combined_accumulated_data();
        let constants = self.to_constant_data();

        KernelCircuitPublicInputs {
            rollup_validation_requests,
            end,
            constants,
            start_state: self.start_state,
            revert_code: self.revert_code,
            fee_payer: self.fee_payer
        }
    }

    pub fn to_kernel_data(self) -> KernelData {
        let public_inputs = self.to_kernel_circuit_public_inputs();
        KernelData {
            public_inputs,
            proof: NestedRecursiveProof::empty(),
            vk: self.vk,
            vk_index: self.vk_index,
            vk_path: self.vk_path
        }
    }

    pub fn add_new_note_hash(&mut self, value: Field) {
        self.note_hashes.push(NoteHash { value, counter: self.next_counter() }.scope(self.storage_contract_address));
    }

    pub fn add_siloed_note_hash(&mut self, value: Field) {
        // First nullifier is tx hash.
        let tx_hash = self.nullifiers.get(0).value();
        let index = self.note_hashes.len();
        let note_hash = NoteHash { value, counter: 0 }.scope(self.storage_contract_address);
        let siloed_value = silo_note_hash(note_hash, tx_hash, index);
        self.add_new_note_hash(siloed_value);
    }

    pub fn append_note_hashes(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                let value = self.mock_note_hash_value(index_offset + i);
                self.add_new_note_hash(value);
            }
        }
    }

    pub fn append_siloed_note_hashes(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                let value = self.mock_note_hash_value(index_offset + i);
                self.add_siloed_note_hash(value);
            }
        }
    }

    pub fn append_note_hashes_with_logs(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                let value = self.mock_note_hash_value(index_offset + i);
                self.add_new_note_hash(value);
                let (log_hash, length) = self.mock_note_encrypted_log(index_offset + i);
                self.add_note_encrypted_log_hash(log_hash, length, self.counter - 1);
            }
        }
    }

    pub fn set_first_nullifier(&mut self) {
        assert_eq(self.nullifiers.len(), 0, "first nullifier already set");
        let value = self.mock_nullifier_value(0);
        let first_nullifier = Nullifier { value, counter: 0, note_hash: 0 }.scope(AztecAddress::zero());
        self.nullifiers.push(first_nullifier);
    }

    pub fn add_nullifier(&mut self, value: Field) {
        self.nullifiers.push(
            Nullifier { value, counter: self.next_counter(), note_hash: 0 }.scope(self.storage_contract_address)
        );
    }

    pub fn add_siloed_nullifier(&mut self, value: Field) {
        let siloed_value = compute_siloed_nullifier(self.storage_contract_address, value);
        self.add_nullifier(siloed_value);
    }

    pub fn append_nullifiers(&mut self, num_extra_nullifier: u32) {
        let index_offset = self.nullifiers.len();
        for i in 0..self.nullifiers.max_len() {
            if i < num_extra_nullifier {
                let value = self.mock_nullifier_value(index_offset + i);
                self.add_nullifier(value);
            }
        }
    }

    pub fn append_siloed_nullifiers(&mut self, num_extra_nullifier: u32) {
        let index_offset = self.nullifiers.len();
        for i in 0..self.nullifiers.max_len() {
            if i < num_extra_nullifier {
                let value = self.mock_nullifier_value(index_offset + i);
                self.add_siloed_nullifier(value);
            }
        }
    }

    pub fn add_l2_to_l1_message(&mut self, content: Field, recipient: EthAddress) {
        self.l2_to_l1_msgs.push(
            L2ToL1Message { recipient, content, counter: self.next_counter() }.scope(self.storage_contract_address)
        );
    }

    pub fn add_siloed_l2_to_l1_message(&mut self, content: Field, recipient: EthAddress) {
        let siloed_content = compute_l2_to_l1_hash(
            self.storage_contract_address,
            recipient,
            content,
            self.tx_context.version,
            self.tx_context.chain_id
        );
        self.add_l2_to_l1_message(siloed_content, recipient);
    }

    pub fn append_l2_to_l1_msgs(&mut self, num: u32) {
        let index_offset = self.l2_to_l1_msgs.len();
        for i in 0..self.l2_to_l1_msgs.max_len() {
            if i < num {
                let (content, recipient) = self.mock_l2_to_l1_msg(index_offset + i);
                self.add_l2_to_l1_message(content, recipient);
            }
        }
    }

    pub fn append_siloed_l2_to_l1_msgs(&mut self, num: u32) {
        let index_offset = self.l2_to_l1_msgs.len();
        for i in 0..self.l2_to_l1_msgs.max_len() {
            if i < num {
                let (content, recipient) = self.mock_l2_to_l1_msg(index_offset + i);
                self.add_siloed_l2_to_l1_message(content, recipient);
            }
        }
    }

    pub fn add_public_data_update_request(&mut self, leaf_slot: Field, value: Field) {
        let update_request = PublicDataUpdateRequest { leaf_slot, new_value: value, counter: self.next_counter() };
        self.public_data_update_requests.push(update_request);
    }

    pub fn append_public_data_update_requests(&mut self, num_updates: u32) {
        let index_offset = self.public_data_update_requests.len();
        for i in 0..self.public_data_update_requests.max_len() {
            if i < num_updates {
                let (leaf_slot, value) = self.mock_public_data_write(index_offset + i);
                self.add_public_data_update_request(leaf_slot, value);
            }
        }
    }

    pub fn append_public_data_read_requests(&mut self, num_reads: u32) {
        let index_offset = self.public_data_reads.len();
        for i in 0..self.public_data_reads.max_len() {
            if i < num_reads {
                let read_request = self.mock_public_data_read(index_offset + i);
                self.public_data_reads.push(read_request);
            }
        }
    }

    pub fn add_read_request_for_pending_note_hash(&mut self, note_hash_index: u32) -> u32 {
        let read_request_index = self.note_hash_read_requests.len();
        let value = self.mock_note_hash_value(note_hash_index);
        let read_request = ReadRequest { value, counter: self.next_counter() }.scope(self.storage_contract_address);
        self.note_hash_read_requests.push(read_request);
        read_request_index
    }

    pub fn append_note_hash_read_requests(&mut self, num_reads: u32) {
        let index_offset = self.note_hash_read_requests.len();
        for i in 0..self.note_hash_read_requests.max_len() {
            if i < num_reads {
                let value = self.mock_note_hash_read_value(index_offset + i);
                let read_request = ReadRequest { value, counter: self.next_counter() }.scope(self.storage_contract_address);
                self.note_hash_read_requests.push(read_request);
            }
        }
    }

    pub fn add_read_request_for_pending_nullifier(&mut self, nullifier_index: u32) -> u32 {
        let read_request_index = self.nullifier_read_requests.len();
        let nullifier = self.mock_nullifier_value(nullifier_index);
        let read_request = ReadRequest { value: nullifier, counter: self.next_counter() }.scope(self.storage_contract_address);
        self.nullifier_read_requests.push(read_request);
        read_request_index
    }

    pub fn add_non_existent_read_request_for_nullifier(&mut self, nullifier: Field) {
        let read_request = ReadRequest { value: nullifier, counter: self.next_counter() }.scope(self.storage_contract_address);
        self.nullifier_non_existent_read_requests.push(read_request);
    }

    pub fn append_nullifier_read_requests(&mut self, num_reads: u32) {
        let index_offset = self.nullifier_read_requests.len();
        for i in 0..self.nullifier_read_requests.max_len() {
            if i < num_reads {
                let value = self.mock_nullifier_read_value(index_offset + i);
                let read_request = ReadRequest { value, counter: self.next_counter() }.scope(self.storage_contract_address);
                self.nullifier_read_requests.push(read_request);
            }
        }
    }

    pub fn add_read_request_for_pending_public_data(&mut self, public_date_update_request_index: u32) -> u32 {
        let new_read_request_index = self.public_data_reads.len();
        let public_write = self.public_data_update_requests.get(public_date_update_request_index);
        let read_request = PublicDataRead { leaf_slot: public_write.leaf_slot, value: public_write.new_value };
        self.public_data_reads.push(read_request);
        new_read_request_index
    }

    pub fn add_request_for_key_validation(&mut self, pk_m: Point, sk_app: Field, sk_app_generator: Field) -> u32 {
        let new_request_index = self.scoped_key_validation_requests_and_generators.len();
        let request = KeyValidationRequest { pk_m, sk_app };
        let request_and_generator = KeyValidationRequestAndGenerator { request, sk_app_generator };
        let scoped_key_validation_request_and_generator = request_and_generator.scope(self.storage_contract_address);
        self.scoped_key_validation_requests_and_generators.push(scoped_key_validation_request_and_generator);

        new_request_index
    }

    pub fn append_key_validation_requests(&mut self, num_requests: u32) {
        let index_offset = self.scoped_key_validation_requests_and_generators.len();
        for i in 0..self.scoped_key_validation_requests_and_generators.max_len() {
            if i < num_requests {
                let request = self.mock_key_validation_request(index_offset + i);
                self.scoped_key_validation_requests_and_generators.push(request.scope(self.storage_contract_address));
            }
        }
    }

    pub fn add_note_encrypted_log_hash(&mut self, value: Field, length: Field, note_hash_counter: u32) {
        let log_hash = NoteLogHash { value, counter: self.next_counter(), length, note_hash_counter };
        self.note_encrypted_logs_hashes.push(log_hash);
        self.encrypted_log_preimages_length += length;
    }

    pub fn append_note_encrypted_log_hashes(&mut self, num: u32) {
        let index_offset = self.note_encrypted_logs_hashes.len();
        for i in 0..self.note_encrypted_logs_hashes.max_len() {
            if i < num {
                let (log_hash, length) = self.mock_note_encrypted_log(index_offset + i);
                self.add_note_encrypted_log_hash(log_hash, length, 0);
            }
        }
    }

    pub fn hash_note_encrypted_log_hashes(&mut self) {
        let log_hashes = self.note_encrypted_logs_hashes.storage.map(|l: NoteLogHash| l.expose_to_public());
        self.note_encrypted_logs_hash = compute_tx_note_logs_hash(log_hashes);
    }

    pub fn add_encrypted_log_hash(&mut self, hash: Field, length: Field) {
        let log_hash = EncryptedLogHash { value: hash, counter: self.next_counter(), length, randomness: 2 };
        self.encrypted_logs_hashes.push(log_hash.scope(self.storage_contract_address));
        self.encrypted_log_preimages_length += length;
    }

    pub fn add_siloed_encrypted_log_hash(&mut self, hash: Field, length: Field) {
        let mut log_hash = EncryptedLogHash { value: hash, counter: self.next_counter(), length, randomness: 2 }.scope(self.storage_contract_address);
        log_hash.log_hash.value = silo_encrypted_log_hash(log_hash);
        self.encrypted_logs_hashes.push(log_hash);
        self.encrypted_log_preimages_length += length;
    }

    pub fn append_encrypted_log_hashes(&mut self, num: u32) {
        let index_offset = self.encrypted_logs_hashes.len();
        for i in 0..self.encrypted_logs_hashes.max_len() {
            if i < num {
                let (log_hash, length) = self.mock_encrypted_log(index_offset + i);
                self.add_encrypted_log_hash(log_hash, length);
            }
        }
    }

    pub fn hash_encrypted_log_hashes(&mut self) {
        let mut log_hashes = self.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.expose_to_public());
        for i in 0..self.encrypted_logs_hashes.max_len() {
            let log_hash = self.encrypted_logs_hashes.storage[i];
            if !log_hash.contract_address.is_zero() {
                log_hashes[i].value = silo_encrypted_log_hash(log_hash);
            }
        }
        self.encrypted_logs_hash = compute_tx_logs_hash(log_hashes);
    }

    pub fn add_unencrypted_log_hash(&mut self, hash: Field, length: Field) {
        let log_hash = LogHash { value: hash, counter: self.next_counter(), length };
        self.unencrypted_logs_hashes.push(log_hash.scope(self.storage_contract_address));
        self.unencrypted_log_preimages_length += length;
    }

    pub fn append_unencrypted_log_hashes(&mut self, num: u32) {
        let index_offset = self.unencrypted_logs_hashes.len();
        for i in 0..self.unencrypted_logs_hashes.max_len() {
            if i < num {
                let (log_hash, length) = self.mock_unencrypted_log(index_offset + i);
                self.add_unencrypted_log_hash(log_hash, length);
            }
        }
    }

    pub fn hash_unencrypted_log_hashes(&mut self) {
        let mut log_hashes = self.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.inner());
        for i in 0..self.unencrypted_logs_hashes.max_len() {
            let log_hash = self.unencrypted_logs_hashes.storage[i];
            if !log_hash.contract_address.is_zero() {
                log_hashes[i].value = silo_unencrypted_log_hash(log_hash);
            }
        }
        self.unencrypted_logs_hash = compute_tx_logs_hash(log_hashes);
    }

    pub fn set_encrypted_logs_hash(&mut self, hash: Field, preimages_length: Field) {
        self.encrypted_logs_hash = hash;
        self.encrypted_log_preimages_length = preimages_length;
    }

    pub fn set_unencrypted_logs_hash(&mut self, hash: Field, preimages_length: Field) {
        self.unencrypted_logs_hash = hash;
        self.unencrypted_log_preimages_length = preimages_length;
    }

    pub fn add_private_call_from_call_stack_item(&mut self, item: PrivateCallStackItem) {
        let start_counter = item.public_inputs.start_side_effect_counter;
        let end_counter = item.public_inputs.end_side_effect_counter;
        self.counter = end_counter + 1;

        self.private_call_requests.push(
            PrivateCallRequest {
            contract_address: item.contract_address,
            call_context: item.public_inputs.call_context,
            args_hash: item.public_inputs.args_hash,
            returns_hash: item.public_inputs.returns_hash,
            start_side_effect_counter: start_counter,
            end_side_effect_counter: end_counter
        }
        );
    }

    pub fn add_private_call_request(&mut self, is_delegate_call: bool) {
        let index_offset = self.private_call_requests.len();
        let mut request = self.mock_private_call_request(index_offset);
        request.call_context = self.generate_call_context(request.contract_address, is_delegate_call);

        let start_counter = self.next_counter();
        let end_counter = start_counter + 10;
        self.counter = end_counter + 1;
        request.start_side_effect_counter = start_counter;
        request.end_side_effect_counter = end_counter;

        self.private_call_requests.push(request);
    }

    pub fn append_private_call_requests(&mut self, num: u32) {
        for i in 0..self.private_call_requests.max_len() {
            if i < num {
                self.add_private_call_request(false);
            }
        }
    }

    pub fn append_private_call_requests_delegate(&mut self, num: u32) {
        for i in 0..self.private_call_requests.max_len() {
            if i < num {
                self.add_private_call_request(true);
            }
        }
    }

    pub fn push_public_call_request(&mut self, item: PublicCallStackItemCompressed) {
        self.public_call_requests.push(PublicCallRequest { item, counter: self.next_counter() });
    }

    pub fn append_public_call_requests(&mut self, num: u32) {
        let index_offset = self.public_call_requests.len();
        for i in 0..self.public_call_requests.max_len() {
            if i < num {
                let item = self.mock_public_call_request_item(index_offset + i, false);
                self.push_public_call_request(item);
            }
        }
    }

    pub fn append_public_call_requests_delegate(&mut self, num: u32) {
        let index_offset = self.public_call_requests.len();
        for i in 0..self.public_call_requests.max_len() {
            if i < num {
                let item = self.mock_public_call_request_item(index_offset + i, true);
                self.push_public_call_request(item);
            }
        }
    }

    pub fn set_fee_payer(&mut self, fee_payer: AztecAddress) {
        self.fee_payer = fee_payer;
    }

    pub fn make_fee_payer(&mut self) -> AztecAddress {
        self.is_fee_payer = true;
        self.set_fee_payer(self.storage_contract_address);
        self.storage_contract_address
    }

    pub fn push_public_teardown_call_request(&mut self, item: PublicCallStackItemCompressed) {
        self.public_teardown_call_stack.push(PublicCallRequest { item, counter: self.next_counter() });
    }

    pub fn append_public_teardown_call_request(&mut self) {
        let index = self.public_teardown_call_stack.len();
        let item = self.mock_public_teardown_call_request_item(index, false);
        self.push_public_teardown_call_request(item);
    }

    pub fn append_public_teardown_call_request_delegate(&mut self) {
        let index = self.public_teardown_call_stack.len();
        let item = self.mock_public_teardown_call_request_item(index, true);
        self.push_public_teardown_call_request(item);
    }

    pub fn end_setup(&mut self) {
        self.min_revertible_side_effect_counter = self.counter;
    }

    pub fn set_max_block_number(&mut self, max_block_number: u32) {
        self.max_block_number = MaxBlockNumber::new(max_block_number);
    }

    fn mock_note_hash_read_value(self, index: u32) -> Field {
        789 + self.value_offset + index as Field
    }

    fn mock_nullifier_read_value(self, index: u32) -> Field {
        22334 + self.value_offset + index as Field
    }

    fn mock_key_validation_request(self, index: u32) -> KeyValidationRequestAndGenerator {
        let value_offset = 3030 + self.value_offset + index as Field;
        let request = KeyValidationRequest { pk_m: Point { x: value_offset, y: 1 + value_offset, is_infinite: false }, sk_app: 2 + value_offset };
        KeyValidationRequestAndGenerator { request, sk_app_generator: 3 + value_offset }
    }

    fn mock_public_data_read(self, index: u32) -> PublicDataRead {
        let value_offset = 4545 + self.value_offset + index as Field;
        PublicDataRead { leaf_slot: value_offset, value: 1 + value_offset }
    }

    fn mock_public_data_write(self, index: u32) -> (Field, Field) {
        let value_offset = 7788 + self.value_offset + index as Field;
        let leaf_slot = value_offset;
        let value = 1 + value_offset;
        (leaf_slot, value)
    }

    fn mock_note_hash_value(self, index: u32) -> Field {
        212121 + self.value_offset + index as Field
    }

    fn mock_nullifier_value(self, index: u32) -> Field {
        5678 + self.value_offset + index as Field
    }

    fn mock_nullifier_value_non_revertible(self, index: u32) -> Field {
        9876 + self.value_offset + index as Field
    }

    fn mock_l2_to_l1_msg(self, index: u32) -> (Field, EthAddress) {
        let value_offset = 72727 + self.value_offset + index as Field;
        (value_offset, EthAddress::from_field(1 + value_offset))
    }

    fn mock_note_encrypted_log(self, index: u32) -> (Field, Field) {
        let log_hash = 282828 + self.value_offset + index as Field;
        let length = 5 + index as Field;
        (log_hash, length)
    }

    fn mock_encrypted_log(self, index: u32) -> (Field, Field) {
        let log_hash = 50403 + self.value_offset + index as Field;
        let length = 3 + index as Field;
        (log_hash, length)
    }

    fn mock_unencrypted_log(self, index: u32) -> (Field, Field) {
        let log_hash = 199199 + self.value_offset + index as Field;
        let length = 1 + index as Field;
        (log_hash, length)
    }

    fn mock_private_call_request(self, index: u32) -> PrivateCallRequest {
        let value_offset = 766766 + self.value_offset + index as Field;
        let mut fields = [0; PRIVATE_CALL_REQUEST_LENGTH];
        for i in 0..fields.len() {
            fields[i] = value_offset + i as Field;
        }
        PrivateCallRequest::deserialize(fields)
    }

    fn mock_public_call_request_item(self, index: u32, is_delegate_call: bool) -> PublicCallStackItemCompressed {
        let value_offset = 636363 + self.value_offset + index as Field;
        let mut fields = [0; PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH];
        for i in 0..fields.len() {
            fields[i] = value_offset + i as Field;
        }
        let mut item = PublicCallStackItemCompressed::deserialize(fields);
        item.call_context = self.generate_call_context(item.contract_address, is_delegate_call);
        item
    }

    fn mock_public_teardown_call_request_item(self, index: u32, is_delegate_call: bool) -> PublicCallStackItemCompressed {
        self.mock_public_call_request_item(index + 54345, is_delegate_call)
    }

    fn mock_fee_payer(self) -> AztecAddress {
        AztecAddress::from_field(900900 + self.value_offset)
    }

    fn generate_call_context(self, target_contract_address: AztecAddress, is_delegate_call: bool) -> CallContext {
        let mut call_context = CallContext::empty();
        call_context.is_delegate_call = is_delegate_call;
        call_context.is_static_call = self.is_static_call;
        if is_delegate_call {
            call_context.msg_sender = self.msg_sender;
            call_context.storage_contract_address = self.storage_contract_address;
        } else {
            call_context.msg_sender = self.storage_contract_address;
            call_context.storage_contract_address = target_contract_address;
        }
        call_context
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.counter;
        self.counter += 1;
        counter
    }

    fn vk_tree_root() -> Field {
        fixtures::vk_tree::get_vk_merkle_tree().get_root()
    }
}

impl Empty for FixtureBuilder {
    fn empty() -> Self {
        FixtureBuilder {
            contract_address: AztecAddress::zero(),
            storage_contract_address: AztecAddress::zero(),
            msg_sender: AztecAddress::zero(),
            is_delegate_call: false,
            is_static_call: false,
            is_fee_payer: false,
            fee_payer: AztecAddress::zero(),
            historical_header: Header::empty(),
            tx_context: TxContext::empty(),
            global_variables: GlobalVariables::empty(),
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            note_encrypted_logs_hash: 0,
            encrypted_logs_hash: 0,
            unencrypted_logs_hash: 0,
            note_encrypted_log_preimages_length: 0,
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            public_data_update_requests: BoundedVec::new(),
            private_call_requests: BoundedVec::new(),
            public_call_requests: BoundedVec::new(),
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            scoped_key_validation_requests_and_generators: BoundedVec::new(),
            public_data_reads: BoundedVec::new(),
            function_data: FunctionData::empty(),
            args_hash: 0,
            returns_hash: 0,
            function_leaf_membership_witness: MembershipWitness::empty(),
            salted_initialization_hash: SaltedInitializationHash::from_field(0),
            public_keys_hash: PublicKeysHash::from_field(0),
            contract_class_artifact_hash: 0,
            contract_class_public_bytecode_commitment: 0,
            acir_hash: 0,
            proof: NestedRecursiveProof::empty(),
            vk: VerificationKey::empty(),
            vk_index: 0,
            vk_path: [0; VK_TREE_HEIGHT],
            vk_tree_root: FixtureBuilder::vk_tree_root(),
            revert_code: 0,
            min_revertible_side_effect_counter: 0,
            counter_start: 0,
            counter: 0,
            start_state: PartialStateReference::empty(),
            gas_used: Gas::empty(),
            non_revertible_gas_used: Gas::empty(),
            public_teardown_call_stack: BoundedVec::new(),
            value_offset: 0,
        }
    }
}
