use crate::{
    transaction::{
        request::TxRequest,
        context::TxContext,
    },
    address::{ Address, EthAddress },
    contrakt::{
        deployment_data::ContractDeploymentData,
    },
    abis::{
        private_kernel::private_call_data::PrivateCallData,
        historical_block_data::HistoricalBlockData,
        call_context::CallContext,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        function_data::FunctionData,
        function_selector::FunctionSelector,
        complete_address::CompleteAddress,
        call_stack_item::{PrivateCallStackItem, CallStackItem},
        membership_witness::{FunctionLeafMembershipWitness, ContractLeafMembershipWitness},
        kernel_circuit_public_inputs::KernelCircuitPublicInputs,
        read_request_membership_witness::ReadRequestMembershipWitness,
    },
    hash::{
        NUM_FIELDS_PER_SHA256,
        compute_constructor_hash,
        compute_partial_address,
        compute_contract_address_from_partial,
        stdlib_recursion_verification_key_compress_native_vk,
    },
    utils::bounded_vec::BoundedVec,
    point::Point,
    block::Block,
    mocked::{ Proof, VerificationKey }
};
use dep::aztec::{
    abi::hash_args,
    constants_gen::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PENDING_READ_REQUESTS_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        RETURN_VALUES_LENGTH,
        NOTE_HASH_TREE_HEIGHT,
    }
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
    call_context : CallContext,
    
    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests : BoundedVec<Field, MAX_READ_REQUESTS_PER_CALL>,
    pending_read_requests : BoundedVec<Field, MAX_PENDING_READ_REQUESTS_PER_CALL>,

    new_commitments : BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers : BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,
    nullified_commitments : BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack : BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

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

            args_hash : inputs.args_hash,
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

pub fn build_tx_request(
    is_constructor: bool, 
    contract_deployment_data: ContractDeploymentData,
    private_call_data: PrivateCallData,
) -> TxRequest {
    TxRequest {
        origin : private_call_data.call_stack_item.inner.contract_address,
        args_hash: private_call_data.call_stack_item.inner.public_inputs.args_hash,
        tx_context : TxContext {
            is_fee_payment_tx : false,
            is_rebate_payment_tx : false,
            is_contract_deployment_tx : is_constructor,
            contract_deployment_data,
            chain_id : 1,
            version: 0,
        },
        function_data : private_call_data.call_stack_item.function_data(),
    }
}

pub fn create_private_call_data<N, T>(
    is_constructor: bool,
    function: fn (PrivateAppInputs, T) -> PrivateCircuitPublicInputs,
    params: T,
    msg_sender: Address,
) -> (PrivateCallData, ContractDeploymentData) {
    let mut contract_address = if is_constructor {
        Address::from_field(0)
    } else {
        Address::from_field(12345)
    };

    let portal_contract_address = EthAddress::from_field(23456);
    let contract_address_salt = 34567;
    let acir_hash = 12341234;
    let args_hash = 234987;
    let private_circuit_vk_hash = 0;

    let msg_sender_pub_key = Point { x: 123456789, y: 123456789 };

    let function_data = FunctionData {
        selector: FunctionSelector::from_u32(1),
        is_private: true,
        is_internal: false,
        is_constructor,
    };

    let mut call_context = CallContext {
        msg_sender,
        storage_contract_address: contract_address,
        portal_contract_address,
        function_selector: function_data.selector,
        is_delegate_call: false,
        is_static_call: false,
        is_contract_deployment: is_constructor,
    };

    let mut contract_deployment_data = ContractDeploymentData{
        deployer_public_key : Point { x: 0, y: 0},
        constructor_vk_hash : 0,
        function_tree_root : 0,
        contract_address_salt : 0,
        portal_contract_address : EthAddress::from_field(0),
    };

    // Constants taken from __snapshots__/noir_test_gen.test.ts.snap
    let function_tree_root = 0x2143c6a86ded93cd75de5680c57c61a2a178e0a840d8649a9e0411032f969f1d;
    let contract_tree_root = 0x13e8827c83abfc4f0a37d0c228fb4ecd509d96613b8e7d74c169005a62ef67dd;
    let function_leaf_sibling_path = [
        0x1f2e3193c7187347a099ee7cb5d6ac077da6b18706fe5508e658a3d0a05494f7,
        0x2350c7d2f19119502d073142ff019af884d6513f38b92d5119b4999502282247,
        0x0837a67313f4dbbd8d6971c0672f961f0a3b9e218c1395d327915209292acbbf,
        0x2e0ef36ddc5db29acb6ef904999046f835ce7c78a40c3f7a0edb03b2f917a765,
    ];
    let contract_leaf_sibling_path = [
        0x00,
        0x27b1d0839a5b23baf12a8d195b18ac288fcf401afb2f70b8a4b529ede5fa9fed,
        0x21dbfd1d029bf447152fcf89e355c334610d1632436ba170f738107266a71550,
        0x0bcd1f91cf7bdd471d0a30c58c4706f3fdab3807a954b8f5b5e3bfec87d001bb,
        0x06e62084ee7b602fe9abc15632dda3269f56fb0c6e12519a2eb2ec897091919d,
        0x03c9e2e67178ac638746f068907e6677b4cc7a9592ef234ab6ab518f17efffa0,
        0x15d28cad4c0736decea8997cb324cf0a0e0602f4d74472cd977bce2c8dd9923f,
        0x268ed1e1c94c3a45a14db4108bc306613a1c23fab68e0466a002dfb0a3f8d2ab,
        0x0cd8d5695bc2dde99dd531671f76f1482f14ddba8eeca7cb9686d4a62359c257,
        0x047fbb7eb974155702149e58ea6ad91f4c6e953e693db35e953e250d8ceac9a9,
        0xc5ae2526e665e2c7c698c11a06098b7159f720606d50e7660deb55758b0b02,
        0x2ced19489ab456b8b6c424594cdbbae59c36dfdd4c4621c4032da2d8a9674be5,
        0x1df5a245ffc1da14b46fe56a605f2a47b1cff1592bab4f66cfe5dfe990af6ab5,
        0x2871d090615d14eadb52228c635c90e0adf31176f0814f6525c23e7d7b318c93,
        0x1a2b85ff013d4b2b25074297c7e44aa61f4836d0862b36db2e6ce2b5542f9ea9,
        0x177b9a10bbee32f77c719c6f8d071a18476cbeb021e155c642bbf93c716ce943,
    ];

    let historical_block_data = HistoricalBlockData {
        blocks_tree_root : 0,
        block : Block {
            note_hash_tree_root : crate::tests::read_requests::ROOT,
            nullifier_tree_root : 0,
            contract_tree_root,
            l1_to_l2_data_tree_root : 0,
            public_data_tree_root : 0,
            global_variables_hash : 0,
        },
        private_kernel_vk_tree_root : 0,
    };

    if is_constructor {
        contract_deployment_data = ContractDeploymentData {
            deployer_public_key: msg_sender_pub_key,
            constructor_vk_hash: private_circuit_vk_hash,
            function_tree_root,
            contract_address_salt,
            portal_contract_address,
        };
        let constructor_hash = compute_constructor_hash(function_data, args_hash, private_circuit_vk_hash);
        contract_address = CompleteAddress::compute(contract_deployment_data.deployer_public_key,
                                                                            contract_deployment_data.contract_address_salt,
                                                                            contract_deployment_data.function_tree_root,
                                                                            constructor_hash).address;
        call_context.storage_contract_address = contract_address;
    }

    let app_inputs = PrivateAppInputs {
        call_context, 
        historical_block_data,
        contract_deployment_data,
        args_hash,
        chain_id: 0,
        version: 1,
    };

    let public_inputs = function(app_inputs, params);

    let call_stack_item = PrivateCallStackItem {
        inner: CallStackItem {
            contract_address,
            public_inputs,
            is_execution_request: false,
            function_data,
        }
    };

    let private_call = PrivateCallData {
        call_stack_item,
        private_call_stack_preimages : dep::std::unsafe::zeroed(),
        proof : Proof {},
        vk : VerificationKey {},
        function_leaf_membership_witness : FunctionLeafMembershipWitness{
            leaf_index : 0,
            sibling_path : function_leaf_sibling_path
        },
        contract_leaf_membership_witness : ContractLeafMembershipWitness {
            leaf_index: 0,
            sibling_path: contract_leaf_sibling_path
        },
        read_request_membership_witnesses : dep::std::unsafe::zeroed(),
        portal_contract_address,
        acir_hash,
    };

    (private_call, contract_deployment_data)
}

pub fn generate_read_requests(how_many: u64) -> ([Field; MAX_READ_REQUESTS_PER_CALL], [ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>; MAX_READ_REQUESTS_PER_CALL]) {
    let mut read_requests = [0; MAX_READ_REQUESTS_PER_CALL];
    let mut read_request_membership_witnesses: [ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>; MAX_READ_REQUESTS_PER_CALL] = dep::std::unsafe::zeroed();

    for i in 0..how_many {
        read_requests[i] = (i as Field) + 1;
        let witness = ReadRequestMembershipWitness {
            leaf_index : i as Field,
            sibling_path : crate::tests::read_requests::SIBLING_PATHS[i],
            is_transient : false,
            hint_to_commitment : 0,
        };
        read_request_membership_witnesses[i] = witness;
    }

    (read_requests, read_request_membership_witnesses)
}

pub fn non_zero_items<N>(array: [Field; N]) -> u64 {
    let mut count = 0;
    for i in 0..N {
        if array[i] != 0 {
            count += 1;
        }
    }
    count
}


pub fn validate_no_new_deployed_contract(public_inputs: KernelCircuitPublicInputs) {
    for new_contract in public_inputs.end.new_contracts {
        assert(new_contract.is_empty());
    }
}

pub fn validate_deployed_contract_address(tx_request: TxRequest, private_call: PrivateCallData, public_inputs: KernelCircuitPublicInputs) {
    assert_eq(public_inputs.end.new_contracts.len(), 1);
    let cdd = tx_request.tx_context.contract_deployment_data;

    let private_circuit_vk_hash = stdlib_recursion_verification_key_compress_native_vk(private_call.vk);
    let constructor_hash = compute_constructor_hash(tx_request.function_data, tx_request.args_hash, private_circuit_vk_hash);
    let contract_address = CompleteAddress::compute(cdd.deployer_public_key,
        cdd.contract_address_salt,
        cdd.function_tree_root,
        constructor_hash).address;
    assert(public_inputs.end.new_contracts[0].contract_address.eq(contract_address));
}