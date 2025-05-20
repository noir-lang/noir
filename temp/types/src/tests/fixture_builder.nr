use crate::{
    abis::{
        accumulated_data::{
            avm_accumulated_data::AvmAccumulatedData, PrivateAccumulatedData,
            PrivateAccumulatedDataBuilder, PrivateToPublicAccumulatedData,
            PrivateToRollupAccumulatedData,
        },
        append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        avm_circuit_public_inputs::AvmProofData,
        call_context::CallContext,
        combined_constant_data::CombinedConstantData,
        constant_rollup_data::ConstantRollupData,
        function_data::FunctionData,
        gas::Gas,
        gas_settings::GasSettings,
        global_variables::GlobalVariables,
        kernel_circuit_public_inputs::{
            PrivateKernelCircuitPublicInputs, PrivateToPublicKernelCircuitPublicInputs,
            PrivateToRollupKernelCircuitPublicInputs,
        },
        log::Log,
        log_hash::LogHash,
        max_block_number::MaxBlockNumber,
        note_hash::{NoteHash, ScopedNoteHash},
        nullifier::{Nullifier, ScopedNullifier},
        private_call_request::PrivateCallRequest,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        private_kernel::private_call_data::{PrivateCallData, PrivateVerificationKeyHints},
        private_kernel_data::PrivateKernelData,
        private_log::{PrivateLog, PrivateLogData},
        protocol_contract_leaf_preimage::ProtocolContractLeafPreimage,
        public_call_request::PublicCallRequest,
        public_data_write::PublicDataWrite,
        public_log::PublicLog,
        read_request::{ReadRequest, ScopedReadRequest},
        side_effect::{Counted, OrderedValue, scoped::Scoped},
        tree_snapshots::TreeSnapshots,
        tube::{PrivateTubeData, PublicTubeData},
        tx_constant_data::TxConstantData,
        validation_requests::{
            KeyValidationRequest, KeyValidationRequestAndGenerator, PrivateValidationRequests,
            RollupValidationRequests, ScopedKeyValidationRequestAndGenerator,
        },
    },
    address::{AztecAddress, EthAddress, SaltedInitializationHash},
    block_header::BlockHeader,
    constants::{
        CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS, DEFAULT_UPDATE_DELAY,
        DEPLOYER_CONTRACT_ADDRESS, FUNCTION_TREE_HEIGHT, MAX_CONTRACT_CLASS_LOGS_PER_TX,
        MAX_ENQUEUED_CALLS_PER_TX, MAX_FIELD_VALUE, MAX_KEY_VALIDATION_REQUESTS_PER_TX,
        MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NOTE_HASHES_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIERS_PER_TX,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, MAX_PRIVATE_LOGS_PER_TX, MAX_PUBLIC_LOGS_PER_TX,
        MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PRIVATE_CALL_REQUEST_LENGTH,
        PRIVATE_LOG_SIZE_IN_FIELDS, PROTOCOL_CONTRACT_TREE_HEIGHT, PUBLIC_CALL_REQUEST_LENGTH,
        PUBLIC_DATA_TREE_HEIGHT, PUBLIC_LOG_SIZE_IN_FIELDS, UPDATED_CLASS_IDS_SLOT, VK_TREE_HEIGHT,
    },
    contract_class_id::ContractClassId,
    data::{
        hash::compute_public_data_tree_index,
        public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
    },
    hash::{
        compute_l2_to_l1_hash, compute_siloed_nullifier, compute_siloed_private_log_field,
        compute_unique_siloed_note_hash, silo_note_hash,
    },
    merkle_tree::{membership::MembershipWitness, MerkleTree},
    messaging::l2_to_l1_message::{L2ToL1Message, ScopedL2ToL1Message},
    partial_state_reference::PartialStateReference,
    point::Point,
    proof::{
        recursive_proof::NestedRecursiveProof,
        verification_key::{ClientIVCVerificationKey, HonkVerificationKey, VerificationKey},
    },
    public_keys::PublicKeys,
    shared_mutable::{
        scheduled_delay_change::ScheduledDelayChange, scheduled_value_change::ScheduledValueChange,
        shared_mutable_values::SharedMutableValues, with_hash::compute_with_hash_hash_storage_slot,
    },
    storage::map::derive_storage_slot_in_map,
    tests::fixtures::{
        self, contract_functions::ContractFunction, contracts::ContractData,
        public_data_tree::empty_public_data_tree,
    },
    traits::{Deserialize, Empty, FromField, Hash, is_empty, Packable, ToField},
    transaction::{tx_context::TxContext, tx_request::TxRequest},
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
            reversed.push(vec.get_unchecked(len - i - 1));
        }
    }
    reversed
}

pub struct FixtureBuilder {
    pub contract_address: AztecAddress,
    pub msg_sender: AztecAddress,
    pub is_static_call: bool,
    pub is_private_only: bool,
    pub claimed_first_nullifier: Field,

    // Fees.
    pub is_fee_payer: bool,
    pub fee_payer: AztecAddress,
    pub public_teardown_call_request: PublicCallRequest,
    pub transaction_fee: Field,
    pub start_gas_left: Gas,
    pub end_gas_left: Gas,

    // Constant data.
    pub historical_header: BlockHeader,
    pub tx_context: TxContext,
    pub global_variables: GlobalVariables,

    // Accumulated data.
    pub note_hashes: BoundedVec<ScopedNoteHash, MAX_NOTE_HASHES_PER_TX>,
    pub nullifiers: BoundedVec<ScopedNullifier, MAX_NULLIFIERS_PER_TX>,
    pub l2_to_l1_msgs: BoundedVec<ScopedL2ToL1Message, MAX_L2_TO_L1_MSGS_PER_TX>,
    pub private_logs: BoundedVec<Scoped<PrivateLogData>, MAX_PRIVATE_LOGS_PER_TX>,
    pub public_logs: BoundedVec<PublicLog, MAX_PUBLIC_LOGS_PER_TX>,
    pub contract_class_logs_hashes: BoundedVec<Scoped<Counted<LogHash>>, MAX_CONTRACT_CLASS_LOGS_PER_TX>,
    pub public_data_writes: BoundedVec<PublicDataWrite, MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    pub private_call_requests: BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    pub public_call_requests: BoundedVec<Counted<PublicCallRequest>, MAX_ENQUEUED_CALLS_PER_TX>,
    pub gas_used: Gas,
    pub revert_code: u8,

    // Validation requests.
    pub max_block_number: MaxBlockNumber,
    pub note_hash_read_requests: BoundedVec<ScopedReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    pub nullifier_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    pub scoped_key_validation_requests_and_generators: BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX>,
    pub validation_requests_split_counter: Option<u32>,

    // Function.
    pub function_data: FunctionData,
    pub args_hash: Field,
    pub calldata_hash: Field,
    pub returns_hash: Field,

    // Private call.
    pub salted_initialization_hash: SaltedInitializationHash,
    pub public_keys: PublicKeys,
    pub contract_class_artifact_hash: Field,
    pub contract_class_public_bytecode_commitment: Field,
    pub function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,

    // Public call.
    pub bytecode_hash: Field,
    pub prover_address: AztecAddress,

    // Proof.
    pub proof: NestedRecursiveProof,
    pub honk_vk: HonkVerificationKey,
    pub client_ivc_vk: ClientIVCVerificationKey,
    pub vk_index: u32,
    pub vk_path: [Field; VK_TREE_HEIGHT],
    pub vk_tree_root: Field,

    // Protocol contracts.
    pub protocol_contract_tree_root: Field,
    pub protocol_contract_membership_witness: MembershipWitness<PROTOCOL_CONTRACT_TREE_HEIGHT>,
    pub protocol_contract_leaf: ProtocolContractLeafPreimage,

    // Contract updates
    pub updated_class_id_witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>,
    pub updated_class_id_leaf: PublicDataTreeLeafPreimage,
    pub updated_class_id_value_change: ScheduledValueChange<ContractClassId>,
    pub updated_class_id_delay_change: ScheduledDelayChange<DEFAULT_UPDATE_DELAY>,

    // Tree snapshots.
    pub archive_tree: AppendOnlyTreeSnapshot,
    pub start_snapshots: TreeSnapshots,
    pub end_snapshots: TreeSnapshots,

    // Counters.
    pub min_revertible_side_effect_counter: u32,
    pub counter_start: u32,
    pub counter: u32,

    // States.
    pub start_state: PartialStateReference,

    // Mock data.
    pub value_offset: Field,
}

impl FixtureBuilder {
    pub fn new() -> Self {
        FixtureBuilder::new_from_counter(0)
    }

    pub fn new_from_counter(counter_start: u32) -> Self {
        let mut builder = FixtureBuilder::empty();

        builder.tx_context = TxContext {
            chain_id: fixtures::CHAIN_ID,
            version: fixtures::VERSION,
            gas_settings: GasSettings::empty(),
        };
        builder.msg_sender = fixtures::contracts::parent_contract.address;
        builder.counter_start = counter_start;
        builder.counter = counter_start + 1;

        let contract_data = fixtures::contracts::default_contract;
        let contract_function = fixtures::contract_functions::default_private_function;

        builder.use_contract(contract_data).use_function(
            contract_function,
            fixtures::contract_functions::default_vk,
        )
    }

    pub fn as_parent_contract(&mut self) -> Self {
        self.contract_address = fixtures::contracts::parent_contract.address;
        self.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn in_vk_tree(&mut self, vk_index: u32) -> Self {
        self.vk_index = vk_index;
        let vk_tree: MerkleTree<fixtures::vk_tree::VK_TREE_WIDTH> =
            fixtures::vk_tree::VK_MERKLE_TREE;

        self.honk_vk = fixtures::vk_tree::generate_fake_honk_vk_for_index(vk_index);
        self.client_ivc_vk = fixtures::vk_tree::generate_fake_client_ivc_vk_for_index(vk_index);

        self.vk_path = vk_tree.get_sibling_path(vk_index);

        self.vk_tree_root = vk_tree.get_root();

        *self
    }

    pub fn vk_tree_root() -> Field {
        fixtures::vk_tree::VK_MERKLE_TREE.get_root()
    }

    pub fn set_protocol_contract_root(&mut self) {
        let (tree, protocol_contract_leaves) =
            fixtures::protocol_contract_tree::get_protocol_contract_tree();
        let contract_address_field = self.contract_address.to_field();
        self.protocol_contract_tree_root = tree.get_root();
        for i in 0..protocol_contract_leaves.len() {
            let leaf = protocol_contract_leaves[i];
            // The below is true if leaf is a valid low_leaf for self.contract_address
            // This is used in validate_contract_address where is_protocol_contract = false.
            if (leaf.address != 0)
                & (leaf.address.lt(contract_address_field))
                & ((contract_address_field.lt(leaf.next_address)) | (leaf.next_address == 0)) {
                self.protocol_contract_leaf = leaf;
                self.protocol_contract_membership_witness = MembershipWitness {
                    leaf_index: i as Field,
                    sibling_path: tree.get_sibling_path(i),
                };
            }
        }
    }

    // contract_index should be the same as contract address.
    pub fn use_protocol_contract(&mut self, contract_index: u32) -> Self {
        let (tree, protocol_contract_leaves) =
            fixtures::protocol_contract_tree::get_protocol_contract_tree();
        self.protocol_contract_tree_root = tree.get_root();
        // This is used in validate_contract_address where is_protocol_contract = true.
        self.protocol_contract_membership_witness = MembershipWitness {
            leaf_index: contract_index as Field,
            sibling_path: tree.get_sibling_path(contract_index),
        };
        self.protocol_contract_leaf = protocol_contract_leaves[contract_index];

        let contract_data = fixtures::contracts::get_protocol_contract(contract_index);
        let function_data =
            fixtures::contract_functions::get_protocol_contract_function(contract_index);

        let _ = self.use_contract(contract_data);
        self.contract_address = AztecAddress::from_field(contract_index as Field);

        self.use_function(function_data, fixtures::contract_functions::default_vk)
    }

    pub fn use_contract(&mut self, contract_data: ContractData) -> Self {
        self.contract_address = contract_data.address;
        self.salted_initialization_hash = contract_data.salted_initialization_hash;
        self.public_keys = contract_data.public_keys;
        self.contract_class_artifact_hash = contract_data.artifact_hash;
        self.contract_class_public_bytecode_commitment = contract_data.public_bytecode_commitment;
        self.set_protocol_contract_root();

        *self
    }

    pub fn use_function(
        &mut self,
        function_data: ContractFunction,
        vk: [Field; CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS],
    ) -> Self {
        self.function_data = function_data.data;
        self.function_leaf_membership_witness = function_data.membership_witness;
        self.client_ivc_vk = VerificationKey { key: vk, hash: function_data.vk_hash };
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

    pub fn to_tx_constant_data(self) -> TxConstantData {
        TxConstantData {
            historical_header: self.historical_header,
            tx_context: self.tx_context,
            vk_tree_root: self.vk_tree_root,
            protocol_contract_tree_root: self.protocol_contract_tree_root,
        }
    }

    pub fn to_constant_data(self) -> CombinedConstantData {
        CombinedConstantData {
            historical_header: self.historical_header,
            tx_context: self.tx_context,
            vk_tree_root: self.vk_tree_root,
            protocol_contract_tree_root: self.protocol_contract_tree_root,
            global_variables: self.global_variables,
        }
    }

    pub fn to_constant_rollup_data(self) -> ConstantRollupData {
        ConstantRollupData {
            last_archive: self.archive_tree,
            vk_tree_root: self.vk_tree_root,
            protocol_contract_tree_root: self.protocol_contract_tree_root,
            global_variables: self.global_variables,
        }
    }

    pub fn build_tx_request(self) -> TxRequest {
        TxRequest {
            origin: self.contract_address,
            args_hash: self.args_hash,
            tx_context: self.tx_context,
            function_data: self.function_data,
        }
    }

    pub fn build_call_context(self) -> CallContext {
        CallContext {
            msg_sender: self.msg_sender,
            contract_address: self.contract_address,
            function_selector: self.function_data.selector,
            is_static_call: self.is_static_call,
        }
    }

    pub fn build_private_call_request(self) -> PrivateCallRequest {
        PrivateCallRequest {
            call_context: self.build_call_context(),
            args_hash: self.args_hash,
            returns_hash: self.returns_hash,
            start_side_effect_counter: self.counter_start,
            end_side_effect_counter: self.counter,
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
            note_hash_read_requests: subarray(self
                .note_hash_read_requests
                .storage()
                .map(|r: ScopedReadRequest| r.read_request)),
            nullifier_read_requests: subarray(self
                .nullifier_read_requests
                .storage()
                .map(|r: ScopedReadRequest| r.read_request)),
            key_validation_requests_and_generators: subarray(self
                .scoped_key_validation_requests_and_generators
                .storage()
                .map(|r: ScopedKeyValidationRequestAndGenerator| r.request)),
            note_hashes: subarray(
                self.note_hashes.storage().map(|n: ScopedNoteHash| n.note_hash),
            ),
            nullifiers: subarray(
                self.nullifiers.storage().map(|n: ScopedNullifier| n.nullifier),
            ),
            private_call_requests: subarray(self.private_call_requests.storage()),
            public_call_requests: subarray(self.public_call_requests.storage()),
            public_teardown_call_request: self.public_teardown_call_request,
            l2_to_l1_msgs: subarray(self.l2_to_l1_msgs.storage().map(|r: ScopedL2ToL1Message| {
                r.message
            })),
            start_side_effect_counter: self.counter_start,
            end_side_effect_counter: self.counter,
            private_logs: subarray(self.private_logs.storage().map(|l: Scoped<PrivateLogData>| {
                l.inner
            })),
            contract_class_logs_hashes: subarray(self
                .contract_class_logs_hashes
                .storage()
                .map(|l: Scoped<Counted<LogHash>>| l.inner)),
            historical_header: self.historical_header,
            tx_context: self.tx_context,
        }
    }

    pub fn to_private_verification_key_hints(self) -> PrivateVerificationKeyHints {
        PrivateVerificationKeyHints {
            function_leaf_membership_witness: self.function_leaf_membership_witness,
            salted_initialization_hash: self.salted_initialization_hash,
            public_keys: self.public_keys,
            contract_class_artifact_hash: self.contract_class_artifact_hash,
            contract_class_public_bytecode_commitment: self
                .contract_class_public_bytecode_commitment,
            protocol_contract_membership_witness: self.protocol_contract_membership_witness,
            protocol_contract_leaf: self.protocol_contract_leaf,
            updated_class_id_witness: self.updated_class_id_witness,
            updated_class_id_leaf: self.updated_class_id_leaf,
            updated_class_id_shared_mutable_values: SharedMutableValues::new(
                self.updated_class_id_value_change,
                self.updated_class_id_delay_change,
            )
                .pack(),
        }
    }

    pub fn compute_update_tree_and_hints(&mut self) {
        let public_data_prefill = 2;
        let mut public_data_tree = empty_public_data_tree::<8, 3>(public_data_prefill);

        if is_empty(self.updated_class_id_value_change)
            & is_empty(self.updated_class_id_delay_change) {
            self.historical_header.state.partial.public_data_tree.root =
                public_data_tree.get_root();
            self.historical_header.state.partial.public_data_tree.next_available_leaf_index =
                public_data_prefill;

            self.updated_class_id_witness = MembershipWitness {
                leaf_index: (public_data_prefill - 1) as Field,
                sibling_path: public_data_tree.get_sibling_path(public_data_prefill - 1),
            };
            self.updated_class_id_leaf = PublicDataTreeLeafPreimage {
                slot: (public_data_prefill - 1) as Field,
                value: 0,
                next_slot: 0,
                next_index: 0,
            };
        } else {
            let hashed_update = SharedMutableValues::new(
                self.updated_class_id_value_change,
                self.updated_class_id_delay_change,
            )
                .hash();
            let shared_mutable_slot =
                derive_storage_slot_in_map(UPDATED_CLASS_IDS_SLOT as Field, self.contract_address);
            let hash_slot = compute_with_hash_hash_storage_slot::<SharedMutableValues<ContractClassId, DEFAULT_UPDATE_DELAY>, _>(
                shared_mutable_slot,
            );
            let hash_leaf_slot =
                compute_public_data_tree_index(DEPLOYER_CONTRACT_ADDRESS, hash_slot);
            public_data_tree.update_leaf(
                public_data_prefill - 1,
                PublicDataTreeLeafPreimage {
                    slot: (public_data_prefill - 1) as Field,
                    value: 0,
                    next_slot: hash_leaf_slot,
                    next_index: public_data_prefill,
                }
                    .hash(),
            );
            self.updated_class_id_leaf = PublicDataTreeLeafPreimage {
                slot: hash_leaf_slot,
                value: hashed_update,
                next_slot: 0,
                next_index: 0,
            };
            public_data_tree.update_leaf(public_data_prefill, self.updated_class_id_leaf.hash());
            self.historical_header.state.partial.public_data_tree.root =
                public_data_tree.get_root();
            self.historical_header.state.partial.public_data_tree.next_available_leaf_index =
                public_data_prefill + 1;
            self.updated_class_id_witness = MembershipWitness {
                leaf_index: public_data_prefill as Field,
                sibling_path: public_data_tree.get_sibling_path(public_data_prefill),
            };
        }
    }

    pub fn set_historical_header_from_call_data(&mut self, private_call: PrivateCallData) {
        self.historical_header = private_call.public_inputs.historical_header;
    }

    pub fn to_private_call_data(mut self) -> PrivateCallData {
        PrivateCallData {
            public_inputs: self.to_private_circuit_public_inputs(),
            vk: self.client_ivc_vk,
            verification_key_hints: self.to_private_verification_key_hints(),
        }
    }

    pub fn to_private_accumulated_data_builder(self) -> PrivateAccumulatedDataBuilder {
        PrivateAccumulatedDataBuilder {
            note_hashes: self.note_hashes,
            nullifiers: self.nullifiers,
            l2_to_l1_msgs: self.l2_to_l1_msgs,
            private_logs: self.private_logs,
            contract_class_logs_hashes: self.contract_class_logs_hashes,
            public_call_requests: self.public_call_requests,
            private_call_stack: vec_reverse(self.private_call_requests),
        }
    }

    pub fn to_private_accumulated_data(self) -> PrivateAccumulatedData {
        self.to_private_accumulated_data_builder().finish()
    }

    pub fn to_public_call_request(self) -> PublicCallRequest {
        PublicCallRequest {
            msg_sender: self.msg_sender,
            contract_address: self.contract_address,
            is_static_call: self.is_static_call,
            calldata_hash: self.calldata_hash,
        }
    }

    pub fn to_private_to_public_accumulated_data(self) -> PrivateToPublicAccumulatedData {
        PrivateToPublicAccumulatedData {
            note_hashes: self.note_hashes.storage().map(|n: ScopedNoteHash| n.value()),
            nullifiers: self.nullifiers.storage().map(|n: ScopedNullifier| n.value()),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage().map(|m: ScopedL2ToL1Message| {
                m.expose_to_public()
            }),
            private_logs: self.private_logs.storage().map(|l: Scoped<PrivateLogData>| {
                l.expose_to_public()
            }),
            contract_class_logs_hashes: self
                .contract_class_logs_hashes
                .storage()
                .map(|l: Scoped<Counted<LogHash>>| l.expose_to_public()),
            public_call_requests: self
                .public_call_requests
                .storage()
                .map(|cr: Counted<PublicCallRequest>| cr.inner),
        }
    }

    pub fn to_private_to_rollup_accumulated_data(self) -> PrivateToRollupAccumulatedData {
        PrivateToRollupAccumulatedData {
            note_hashes: self.note_hashes.storage().map(|n: ScopedNoteHash| n.note_hash.value),
            nullifiers: self.nullifiers.storage().map(|n: ScopedNullifier| n.nullifier.value),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage().map(|m: ScopedL2ToL1Message| {
                m.expose_to_public()
            }),
            private_logs: self.private_logs.storage().map(|l: Scoped<PrivateLogData>| {
                l.expose_to_public()
            }),
            contract_class_logs_hashes: self
                .contract_class_logs_hashes
                .storage()
                .map(|l: Scoped<Counted<LogHash>>| l.expose_to_public()),
        }
    }

    pub fn to_private_validation_requests(self) -> PrivateValidationRequests {
        PrivateValidationRequests {
            for_rollup: self.to_rollup_validation_requests(),
            note_hash_read_requests: self.note_hash_read_requests.storage(),
            nullifier_read_requests: self.nullifier_read_requests.storage(),
            scoped_key_validation_requests_and_generators: self
                .scoped_key_validation_requests_and_generators
                .storage(),
            split_counter: self.validation_requests_split_counter,
        }
    }

    pub fn to_private_kernel_circuit_public_inputs(self) -> PrivateKernelCircuitPublicInputs {
        let constants = self.to_tx_constant_data();
        let end = self.to_private_accumulated_data();
        let validation_requests = self.to_private_validation_requests();
        let public_teardown_call_request = self.public_teardown_call_request;

        PrivateKernelCircuitPublicInputs {
            constants,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            end,
            validation_requests,
            public_teardown_call_request,
            fee_payer: self.fee_payer,
            is_private_only: self.is_private_only,
            claimed_first_nullifier: self.claimed_first_nullifier,
        }
    }

    pub fn to_private_kernel_data(self) -> PrivateKernelData {
        let public_inputs = self.to_private_kernel_circuit_public_inputs();
        PrivateKernelData {
            public_inputs,
            vk: self.client_ivc_vk,
            vk_index: self.vk_index,
            vk_path: self.vk_path,
        }
    }

    pub fn to_private_to_public_kernel_circuit_public_inputs(
        self,
        revertible: bool,
    ) -> PrivateToPublicKernelCircuitPublicInputs {
        // TODO: Split the data using self.min_revertible_side_effect_counter.
        let accumulated_data = self.to_private_to_public_accumulated_data();
        let non_revertible_accumulated_data = if revertible {
            PrivateToPublicAccumulatedData::empty()
        } else {
            accumulated_data
        };
        let revertible_accumulated_data = if revertible {
            accumulated_data
        } else {
            PrivateToPublicAccumulatedData::empty()
        };

        PrivateToPublicKernelCircuitPublicInputs {
            constants: self.to_tx_constant_data(),
            rollup_validation_requests: self.to_rollup_validation_requests(),
            non_revertible_accumulated_data,
            revertible_accumulated_data,
            public_teardown_call_request: self.public_teardown_call_request,
            gas_used: self.gas_used,
            fee_payer: self.fee_payer,
        }
    }

    pub fn to_rollup_validation_requests(self) -> RollupValidationRequests {
        RollupValidationRequests { max_block_number: self.max_block_number }
    }

    pub fn to_private_to_rollup_kernel_circuit_public_inputs(
        self,
    ) -> PrivateToRollupKernelCircuitPublicInputs {
        let rollup_validation_requests = self.to_rollup_validation_requests();
        let end = self.to_private_to_rollup_accumulated_data();
        let constants = self.to_tx_constant_data();

        PrivateToRollupKernelCircuitPublicInputs {
            rollup_validation_requests,
            end,
            constants,
            gas_used: self.gas_used,
            fee_payer: self.fee_payer,
        }
    }

    pub fn to_private_tube_data(self) -> PrivateTubeData {
        let mut result: PrivateTubeData = std::mem::zeroed();
        result.public_inputs = self.to_private_to_rollup_kernel_circuit_public_inputs();
        result
    }

    pub fn to_public_tube_data(self) -> PublicTubeData {
        let mut result: PublicTubeData = std::mem::zeroed();
        result.public_inputs = self.to_private_to_public_kernel_circuit_public_inputs(true);
        result
    }

    pub fn to_avm_accumulated_data(self) -> AvmAccumulatedData {
        AvmAccumulatedData {
            note_hashes: self.note_hashes.storage().map(|n: ScopedNoteHash| n.note_hash.value),
            nullifiers: self.nullifiers.storage().map(|n: ScopedNullifier| n.nullifier.value),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage(),
            public_logs: self.public_logs.storage(),
            public_data_writes: self.public_data_writes.storage(),
        }
    }

    pub fn to_avm_proof_data(self, reverted: bool) -> AvmProofData {
        let mut result: AvmProofData = std::mem::zeroed();

        result.public_inputs.reverted = reverted;
        result.public_inputs.global_variables = self.global_variables;
        result.public_inputs.accumulated_data = self.to_avm_accumulated_data();
        result.public_inputs.start_tree_snapshots = self.start_snapshots;
        result.public_inputs.end_tree_snapshots = self.end_snapshots;

        result
    }

    pub fn add_new_note_hash(&mut self, value: Field) {
        self.note_hashes.push(NoteHash { value, counter: self.next_counter() }.scope(
            self.contract_address,
        ));
    }

    pub fn add_siloed_note_hash(&mut self, value: Field) {
        let note_hash_to_silo = NoteHash { value, counter: 0 }.scope(self.contract_address);
        let siloed_value = silo_note_hash(note_hash_to_silo);
        self.note_hashes.push(NoteHash { value: siloed_value, counter: self.next_counter() }.scope(
            AztecAddress::zero(),
        ));
    }

    pub fn add_siloed_unique_note_hash(&mut self, value: Field) {
        let note_hash_to_silo = NoteHash { value, counter: 0 }.scope(self.contract_address);
        let siloed_value = silo_note_hash(note_hash_to_silo);
        let current_index = self.note_hashes.len();
        let unique_siloed_value = compute_unique_siloed_note_hash(
            siloed_value,
            self.claimed_first_nullifier,
            current_index,
        );

        self.note_hashes.push(NoteHash { value: unique_siloed_value, counter: self.next_counter() }
            .scope(AztecAddress::zero()));
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

    pub fn append_siloed_unique_note_hashes(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                let value = self.mock_note_hash_value(index_offset + i);
                self.add_siloed_unique_note_hash(value);
            }
        }
    }

    pub fn append_note_hashes_with_logs(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                let value = self.mock_note_hash_value(index_offset + i);
                self.add_new_note_hash(value);
                self.append_private_logs_for_note(1, self.counter - 1);
            }
        }
    }

    pub fn set_protocol_nullifier(&mut self) {
        assert_eq(self.nullifiers.len(), 0, "first nullifier already set");
        let value = self.mock_nullifier_value(0);
        let first_nullifier =
            Nullifier { value, counter: 0, note_hash: 0 }.scope(AztecAddress::zero());
        self.nullifiers.push(first_nullifier);
        self.claimed_first_nullifier = value;
    }

    pub fn add_nullifier(&mut self, value: Field) {
        let note_hash = 0;
        self.add_nullifier_for_note_hash(value, note_hash);
    }

    pub fn add_nullifier_for_note_hash(&mut self, value: Field, note_hash: Field) {
        self.nullifiers.push(Nullifier { value, counter: self.next_counter(), note_hash }.scope(
            self.contract_address,
        ));
    }

    pub fn add_siloed_nullifier(&mut self, value: Field) {
        let note_hash = 0;
        self.add_siloed_nullifier_for_note_hash(value, note_hash);
    }

    pub fn add_siloed_nullifier_for_note_hash(&mut self, value: Field, note_hash: Field) {
        let siloed_value = compute_siloed_nullifier(self.contract_address, value);
        self.nullifiers.push(Nullifier {
            value: siloed_value,
            counter: self.next_counter(),
            note_hash,
        }
            .scope(AztecAddress::zero()));
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
        self.l2_to_l1_msgs.push(L2ToL1Message { recipient, content, counter: self.next_counter() }
            .scope(self.contract_address));
    }

    pub fn add_exposed_l2_to_l1_message(&mut self, content: Field, recipient: EthAddress) {
        self.l2_to_l1_msgs.push(L2ToL1Message { recipient, content, counter: 0 }.scope(
            self.contract_address,
        ));
    }

    pub fn add_siloed_l2_to_l1_message(&mut self, content: Field, recipient: EthAddress) {
        let siloed_content = compute_l2_to_l1_hash(
            self.contract_address,
            recipient,
            content,
            self.tx_context.version,
            self.tx_context.chain_id,
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

    pub fn append_exposed_l2_to_l1_msgs(&mut self, num: u32) {
        let index_offset = self.l2_to_l1_msgs.len();
        for i in 0..self.l2_to_l1_msgs.max_len() {
            if i < num {
                let (content, recipient) = self.mock_l2_to_l1_msg(index_offset + i);
                self.add_exposed_l2_to_l1_message(content, recipient);
            }
        }
    }

    pub fn add_read_request_for_pending_note_hash(&mut self, note_hash_index: u32) -> u32 {
        let read_request_index = self.note_hash_read_requests.len();
        let value = self.mock_note_hash_value(note_hash_index);
        let read_request =
            ReadRequest { value, counter: self.next_counter() }.scope(self.contract_address);
        self.note_hash_read_requests.push(read_request);
        read_request_index
    }

    pub fn append_note_hash_read_requests(&mut self, num_reads: u32) {
        let index_offset = self.note_hash_read_requests.len();
        for i in 0..self.note_hash_read_requests.max_len() {
            if i < num_reads {
                let value = self.mock_note_hash_read_value(index_offset + i);
                let read_request = ReadRequest { value, counter: self.next_counter() }.scope(
                    self.contract_address,
                );
                self.note_hash_read_requests.push(read_request);
            }
        }
    }

    pub fn add_read_request_for_pending_nullifier(&mut self, nullifier_index: u32) -> u32 {
        let read_request_index = self.nullifier_read_requests.len();
        let nullifier = self.mock_nullifier_value(nullifier_index);
        let read_request = ReadRequest { value: nullifier, counter: self.next_counter() }.scope(
            self.contract_address,
        );
        self.nullifier_read_requests.push(read_request);
        read_request_index
    }

    pub fn append_nullifier_read_requests(&mut self, num_reads: u32) {
        let index_offset = self.nullifier_read_requests.len();
        for i in 0..self.nullifier_read_requests.max_len() {
            if i < num_reads {
                let value = self.mock_nullifier_read_value(index_offset + i);
                let read_request = ReadRequest { value, counter: self.next_counter() }.scope(
                    self.contract_address,
                );
                self.nullifier_read_requests.push(read_request);
            }
        }
    }

    pub fn add_request_for_key_validation(
        &mut self,
        pk_m: Point,
        sk_app: Field,
        sk_app_generator: Field,
    ) -> u32 {
        let new_request_index = self.scoped_key_validation_requests_and_generators.len();
        let request = KeyValidationRequest { pk_m, sk_app };
        let request_and_generator = KeyValidationRequestAndGenerator { request, sk_app_generator };
        let scoped_key_validation_request_and_generator =
            request_and_generator.scope(self.contract_address);
        self.scoped_key_validation_requests_and_generators.push(
            scoped_key_validation_request_and_generator,
        );

        new_request_index
    }

    pub fn append_key_validation_requests(&mut self, num_requests: u32) {
        let index_offset = self.scoped_key_validation_requests_and_generators.len();
        for i in 0..self.scoped_key_validation_requests_and_generators.max_len() {
            if i < num_requests {
                let request = self.mock_key_validation_request(index_offset + i);
                self.scoped_key_validation_requests_and_generators.push(request.scope(
                    self.contract_address,
                ));
            }
        }
    }

    pub fn add_private_log(
        &mut self,
        fields: [Field; PRIVATE_LOG_SIZE_IN_FIELDS],
        length: u32,
        note_hash_counter: u32,
    ) {
        let log = PrivateLog { fields, length };
        let logData = PrivateLogData { log, note_hash_counter, counter: self.next_counter() }.scope(
            self.contract_address,
        );
        self.private_logs.push(logData);
    }

    pub fn append_private_logs_for_note(&mut self, num_logs: u32, note_hash_counter: u32) {
        let index_offset = self.private_logs.len();
        for i in 0..self.private_logs.max_len() {
            if i < num_logs {
                let fields = self.mock_private_log_fields(index_offset + i);
                self.add_private_log(fields, fields.len(), note_hash_counter);
            }
        }
    }

    pub fn append_private_logs(&mut self, num_logs: u32) {
        self.append_private_logs_for_note(num_logs, 0 /* note_hash_counter */);
    }

    pub fn append_private_logs_with_lengths<let N: u32>(&mut self, lengths: [u32; N]) {
        let index_offset = self.private_logs.len();
        for i in 0..lengths.len() {
            let fields = self.mock_private_log_fields_with_length(index_offset + i, lengths[i]);
            self.add_private_log(fields, lengths[i], 0 /* note_hash_counter */);
        }
    }

    pub fn add_siloed_private_log(
        &mut self,
        fields: [Field; PRIVATE_LOG_SIZE_IN_FIELDS],
        length: u32,
        note_hash_counter: u32,
    ) {
        let log = PrivateLogData {
            log: PrivateLog { fields, length },
            note_hash_counter,
            counter: self.next_counter(),
        }
            .scope(AztecAddress::zero()); // When siloed, the address is set to zero by the reset kernel.
        self.private_logs.push(log);
    }

    pub fn append_siloed_private_logs_for_note(&mut self, num_logs: u32, note_hash_counter: u32) {
        let index_offset = self.private_logs.len();
        for i in 0..self.private_logs.max_len() {
            if i < num_logs {
                let mut fields = self.mock_private_log_fields(index_offset + i);
                fields[0] = compute_siloed_private_log_field(self.contract_address, fields[0]);
                self.add_siloed_private_log(fields, fields.len(), note_hash_counter);
            }
        }
    }

    pub fn append_siloed_private_logs_for_note_with_lengths<let N: u32>(
        &mut self,
        lengths: [u32; N],
        note_hash_counter: u32,
    ) {
        let index_offset = self.private_logs.len();
        for i in 0..lengths.len() {
            let length = lengths[i];
            let mut fields = self.mock_private_log_fields_with_length(index_offset + i, length);
            fields[0] = compute_siloed_private_log_field(self.contract_address, fields[0]);
            self.add_siloed_private_log(fields, length, note_hash_counter);
        }
    }

    pub fn add_public_log(&mut self, fields: [Field; PUBLIC_LOG_SIZE_IN_FIELDS], length: u32) {
        let inner = Log { fields, length };
        let log = PublicLog { log: inner, contract_address: self.contract_address };
        let _ = self.next_counter();
        self.public_logs.push(log);
    }

    pub fn append_public_logs(&mut self, num_logs: u32) {
        let index_offset = self.public_logs.len();
        for i in 0..self.public_logs.max_len() {
            if i < num_logs {
                let fields = self.mock_public_log_fields(index_offset + i);
                self.add_public_log(fields, fields.len());
            }
        }
    }

    pub fn append_public_logs_with_lengths<let N: u32>(&mut self, lengths: [u32; N]) {
        let index_offset = self.public_logs.len();
        for i in 0..lengths.len() {
            let length = lengths[i];
            let fields = self.mock_public_log_fields_with_length(index_offset + i, length);
            self.add_public_log(fields, length);
        }
    }

    pub fn add_contract_class_log_hash(&mut self, hash: Field, length: u32) {
        let log_hash =
            LogHash { value: hash, length }.count(self.next_counter()).scope(self.contract_address);
        self.contract_class_logs_hashes.push(log_hash);
    }

    pub fn add_private_call_request_for_private_call(&mut self, private_call: PrivateCallData) {
        let public_inputs = private_call.public_inputs;
        let start_counter = public_inputs.start_side_effect_counter;
        let end_counter = public_inputs.end_side_effect_counter;
        self.counter = end_counter + 1;

        self.private_call_requests.push(
            PrivateCallRequest {
                call_context: public_inputs.call_context,
                args_hash: public_inputs.args_hash,
                returns_hash: public_inputs.returns_hash,
                start_side_effect_counter: start_counter,
                end_side_effect_counter: end_counter,
            },
        );
    }

    pub fn add_private_call_request(&mut self) {
        let index_offset = self.private_call_requests.len();
        let mut request = self.mock_private_call_request(index_offset);

        let start_counter = self.next_counter();
        let end_counter = start_counter + 10;
        request.start_side_effect_counter = start_counter;
        request.end_side_effect_counter = end_counter;
        self.counter = end_counter + 1;

        self.private_call_requests.push(request);
    }

    pub fn append_private_call_requests(&mut self, num: u32) {
        for i in 0..self.private_call_requests.max_len() {
            if i < num {
                self.add_private_call_request();
            }
        }
    }

    pub fn add_public_call_request(&mut self, request: PublicCallRequest) {
        self.public_call_requests.push(Counted::new(request, self.next_counter()));
    }

    pub fn append_public_call_requests(&mut self, num: u32) {
        let index_offset = self.public_call_requests.len();
        for i in 0..self.public_call_requests.max_len() {
            if i < num {
                let request = self.mock_public_call_request(index_offset + i);
                self.add_public_call_request(request);
            }
        }
    }

    pub fn set_fee_payer(&mut self, fee_payer: AztecAddress) {
        self.fee_payer = fee_payer;
    }

    pub fn make_fee_payer(&mut self) -> AztecAddress {
        self.is_fee_payer = true;
        self.set_fee_payer(self.contract_address);
        self.contract_address
    }

    pub fn set_gas_used(&mut self, da_gas: u32, l2_gas: u32) {
        self.gas_used = Gas { da_gas, l2_gas };
    }

    pub fn set_public_teardown_call_request(&mut self) {
        self.public_teardown_call_request = self.mock_public_teardown_call_request();
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
        let request = KeyValidationRequest {
            pk_m: Point { x: value_offset, y: 1 + value_offset, is_infinite: false },
            sk_app: 2 + value_offset,
        };
        KeyValidationRequestAndGenerator { request, sk_app_generator: 3 + value_offset }
    }

    fn mock_public_data_read(self, index: u32) -> (Field, Field) {
        let value_offset = 4545 + self.value_offset + index as Field;
        (value_offset, value_offset + 1)
    }

    fn mock_public_data_write(self, index: u32) -> (Field, Field) {
        let value_offset = 7788 + self.value_offset + index as Field;
        let leaf_slot = value_offset;
        let value = 1 + value_offset;
        (leaf_slot, value)
    }

    fn mock_contract_storage_read(self, index: u32) -> (Field, Field) {
        let value_offset = 543543 + self.value_offset + index as Field;
        (value_offset, value_offset + 1)
    }

    fn mock_contract_storage_write(self, index: u32) -> (Field, Field) {
        let value_offset = 336699 + self.value_offset + index as Field;
        (value_offset, value_offset + 1)
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

    fn mock_private_log_fields(self, index: u32) -> [Field; PRIVATE_LOG_SIZE_IN_FIELDS] {
        self.mock_private_log_fields_with_length(index, PRIVATE_LOG_SIZE_IN_FIELDS)
    }

    fn mock_private_log_fields_with_length(
        self,
        index: u32,
        length: u32,
    ) -> [Field; PRIVATE_LOG_SIZE_IN_FIELDS] {
        let value_offset =
            328732 + self.value_offset + (index * PRIVATE_LOG_SIZE_IN_FIELDS) as Field;
        let mut fields = [0; PRIVATE_LOG_SIZE_IN_FIELDS];
        for i in 0..PRIVATE_LOG_SIZE_IN_FIELDS {
            if i < length {
                fields[i] = value_offset + i as Field;
            }
        }
        fields
    }

    fn mock_public_log_fields(self, index: u32) -> [Field; PUBLIC_LOG_SIZE_IN_FIELDS] {
        self.mock_public_log_fields_with_length(index, PUBLIC_LOG_SIZE_IN_FIELDS)
    }

    fn mock_public_log_fields_with_length(
        self,
        index: u32,
        length: u32,
    ) -> [Field; PUBLIC_LOG_SIZE_IN_FIELDS] {
        let value_offset =
            199199 + self.value_offset + (index * PUBLIC_LOG_SIZE_IN_FIELDS) as Field;
        let mut fields = [0; PUBLIC_LOG_SIZE_IN_FIELDS];
        for i in 0..PUBLIC_LOG_SIZE_IN_FIELDS {
            if i < length {
                fields[i] = value_offset + i as Field;
            }
        }
        fields
    }

    fn mock_private_log_randomness(self, index: u32) -> Field {
        579579 + self.value_offset + index as Field
    }

    fn mock_private_call_request(self, index: u32) -> PrivateCallRequest {
        let value_offset = 766766 + self.value_offset + index as Field;
        let mut fields = [0; PRIVATE_CALL_REQUEST_LENGTH];
        for i in 0..fields.len() {
            fields[i] = value_offset + i as Field;
        }
        let mut request = PrivateCallRequest::deserialize(fields);
        request.call_context.msg_sender = self.contract_address;
        request.call_context.is_static_call = self.is_static_call;
        request
    }

    fn mock_public_call_request(self, index: u32) -> PublicCallRequest {
        let value_offset = 636363 + self.value_offset + index as Field;
        let mut fields = [0; PUBLIC_CALL_REQUEST_LENGTH];
        for i in 0..fields.len() {
            fields[i] = value_offset + i as Field;
        }
        let mut request = PublicCallRequest::deserialize(fields);
        request.msg_sender = self.contract_address;
        request.is_static_call = self.is_static_call;
        request
    }

    fn mock_public_teardown_call_request(self) -> PublicCallRequest {
        self.mock_public_call_request(54345)
    }

    fn mock_fee_payer(self) -> AztecAddress {
        AztecAddress::from_field(900900 + self.value_offset)
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

impl Empty for FixtureBuilder {
    fn empty() -> Self {
        FixtureBuilder {
            contract_address: AztecAddress::zero(),
            msg_sender: AztecAddress::zero(),
            is_private_only: false,
            claimed_first_nullifier: 0,
            is_static_call: false,
            is_fee_payer: false,
            fee_payer: AztecAddress::zero(),
            public_teardown_call_request: PublicCallRequest::empty(),
            historical_header: BlockHeader::empty(),
            tx_context: TxContext::empty(),
            global_variables: GlobalVariables::empty(),
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            l2_to_l1_msgs: BoundedVec::new(),
            private_logs: BoundedVec::new(),
            public_logs: BoundedVec::new(),
            contract_class_logs_hashes: BoundedVec::new(),
            public_data_writes: BoundedVec::new(),
            private_call_requests: BoundedVec::new(),
            public_call_requests: BoundedVec::new(),
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            scoped_key_validation_requests_and_generators: BoundedVec::new(),
            validation_requests_split_counter: Option::none(),
            function_data: FunctionData::empty(),
            args_hash: 0,
            calldata_hash: 0,
            returns_hash: 0,
            function_leaf_membership_witness: MembershipWitness::empty(),
            salted_initialization_hash: SaltedInitializationHash::from_field(0),
            public_keys: PublicKeys::default(),
            contract_class_artifact_hash: 0,
            contract_class_public_bytecode_commitment: 0,
            bytecode_hash: 0,
            prover_address: AztecAddress::zero(),
            proof: NestedRecursiveProof::empty(),
            honk_vk: VerificationKey::empty(),
            client_ivc_vk: VerificationKey::empty(),
            vk_index: 0,
            vk_path: [0; VK_TREE_HEIGHT],
            vk_tree_root: FixtureBuilder::vk_tree_root(),
            protocol_contract_tree_root: 0,
            protocol_contract_membership_witness: MembershipWitness::empty(),
            protocol_contract_leaf: ProtocolContractLeafPreimage::empty(),
            updated_class_id_witness: MembershipWitness::empty(),
            updated_class_id_leaf: PublicDataTreeLeafPreimage::empty(),
            updated_class_id_value_change: ScheduledValueChange::empty(),
            updated_class_id_delay_change: ScheduledDelayChange::empty(),
            archive_tree: AppendOnlyTreeSnapshot::zero(),
            start_snapshots: TreeSnapshots::empty(),
            end_snapshots: TreeSnapshots::empty(),
            revert_code: 0,
            min_revertible_side_effect_counter: 0,
            counter_start: 0,
            counter: 0,
            start_state: PartialStateReference::empty(),
            gas_used: Gas::empty(),
            start_gas_left: Gas::empty(),
            end_gas_left: Gas::empty(),
            transaction_fee: 0,
            value_offset: 0,
        }
    }
}
