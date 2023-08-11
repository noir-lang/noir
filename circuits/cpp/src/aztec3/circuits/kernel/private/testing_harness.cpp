#include "testing_harness.hpp"

#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/call_context.hpp"
#include "aztec3/circuits/abis/call_stack_item.hpp"
#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/combined_constant_data.hpp"
#include "aztec3/circuits/abis/constant_historic_block_data.hpp"
#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/abis/tx_context.hpp"
#include "aztec3/circuits/abis/tx_request.hpp"
#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

#include <cstdint>
#include <memory>
#include <vector>

namespace aztec3::circuits::kernel::private_kernel::testing_harness {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CombinedAccumulatedData;
using aztec3::circuits::abis::CombinedConstantData;
using aztec3::circuits::abis::ConstantHistoricBlockData;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateTypes;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::private_kernel::PrivateCallData;

using aztec3::utils::array_length;

/**
 * @brief Get the random read requests and their membership requests
 *
 * @details read requests are siloed by contract address and nonce before being
 * inserted into mock private data tree
 *
 * @param first_nullifier used when computing nonce for unique_siloed_commitments (private data tree leaves)
 * @param contract_address address to use when siloing read requests
 * @param num_read_requests if negative, use random num. Must be < MAX_READ_REQUESTS_PER_CALL
 * @return std::tuple<read_requests, read_request_memberships_witnesses, historic_private_data_tree_root>
 */
std::tuple<std::array<NT::fr, MAX_READ_REQUESTS_PER_CALL>,
           std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>,
           std::array<NT::fr, MAX_READ_REQUESTS_PER_CALL>,
           std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>,
           NT::fr>
get_random_reads(NT::fr const& first_nullifier, NT::fr const& contract_address, int const num_read_requests)
{
    std::array<fr, MAX_READ_REQUESTS_PER_CALL> transient_read_requests{};
    std::array<fr, MAX_READ_REQUESTS_PER_CALL> read_requests{};
    std::array<fr, MAX_READ_REQUESTS_PER_CALL> leaves{};

    // randomize the number of read requests with a configurable minimum
    const auto final_num_rr = num_read_requests >= 0
                                  ? std::min(static_cast<size_t>(num_read_requests), MAX_READ_REQUESTS_PER_CALL)
                                  : numeric::random::get_engine().get_random_uint8() % (MAX_READ_REQUESTS_PER_CALL + 1);
    // randomize private app circuit's read requests
    for (size_t rr = 0; rr < final_num_rr; rr++) {
        // randomize commitment and its leaf index
        // transient read requests are raw (not siloed and not unique at input to kernel circuit)
        transient_read_requests[rr] = NT::fr::random_element();

        const auto siloed_commitment = silo_commitment<NT>(contract_address, read_requests[rr]);
        const auto nonce = compute_commitment_nonce<NT>(first_nullifier, rr);
        const auto unique_siloed_commitment =
            siloed_commitment == 0 ? 0 : compute_unique_commitment<NT>(nonce, siloed_commitment);

        leaves[rr] = unique_siloed_commitment;
        read_requests[rr] = unique_siloed_commitment;
    }

    // this set and the following loop lets us generate totally random leaf indices
    // for read requests while avoiding collisions
    std::unordered_set<NT::uint32> rr_leaf_indices_set;
    while (rr_leaf_indices_set.size() < final_num_rr) {
        rr_leaf_indices_set.insert(numeric::random::get_engine().get_random_uint32() % PRIVATE_DATA_TREE_NUM_LEAVES);
    }
    // set -> vector without collisions
    std::vector<NT::uint32> rr_leaf_indices(rr_leaf_indices_set.begin(), rr_leaf_indices_set.end());

    MemoryStore private_data_tree_store;
    MerkleTree private_data_tree = MerkleTree(private_data_tree_store, PRIVATE_DATA_TREE_HEIGHT);

    // add the commitments to the private data tree for each read request
    // add them at their corresponding index in the tree
    // (in practice the the tree is left-to-right append-only, but here
    // we treat it as sparse just to get these commitments in their correct spot)
    for (size_t i = 0; i < array_length(leaves); i++) {
        private_data_tree.update_element(rr_leaf_indices[i], leaves[i]);
    }

    // compute the merkle sibling paths for each request
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>
        transient_read_request_membership_witnesses{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>
        read_request_membership_witnesses{};
    for (size_t i = 0; i < array_length(read_requests); i++) {
        read_request_membership_witnesses[i] = { .leaf_index = NT::fr(rr_leaf_indices[i]),
                                                 .sibling_path = get_sibling_path<PRIVATE_DATA_TREE_HEIGHT>(
                                                     private_data_tree, rr_leaf_indices[i], 0),
                                                 .is_transient = false,
                                                 .hint_to_commitment = 0 };
        transient_read_request_membership_witnesses[i] = {
            .leaf_index = NT::fr(0),
            .sibling_path = compute_empty_sibling_path<NT, PRIVATE_DATA_TREE_HEIGHT>(0),
            .is_transient = true,
            .hint_to_commitment = 0
        };
    }


    return { read_requests,
             read_request_membership_witnesses,
             transient_read_requests,
             transient_read_request_membership_witnesses,
             private_data_tree.root() };
}  // namespace aztec3::circuits::kernel::private_kernel::testing_harness

std::pair<PrivateCallData<NT>, ContractDeploymentData<NT>> create_private_call_deploy_data(
    bool const is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    NT::address const& msg_sender,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash,
    NT::fr const& encrypted_log_preimages_length,
    NT::fr const& unencrypted_log_preimages_length,
    bool is_circuit)
{
    //***************************************************************************
    // Initialize some inputs to private call and kernel circuits
    //***************************************************************************
    // TODO(suyash) randomize inputs
    NT::address contract_address = is_constructor ? 0 : 12345;  // updated later if in constructor
    const NT::uint32 contract_leaf_index = 1;
    const NT::uint32 function_leaf_index = 1;
    const NT::fr portal_contract_address = 23456;
    const NT::fr contract_address_salt = 34567;
    const NT::fr acir_hash = 12341234;

    const NT::fr msg_sender_private_key = 123456789;
    const Point<NT> msg_sender_pub_key = { .x = 123456789, .y = 123456789 };

    FunctionData<NT> const function_data{
        .function_selector = 1,  // TODO(suyash): deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = is_constructor,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .portal_contract_address = portal_contract_address,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = is_constructor,
    };

    // sometimes need private call args as array
    std::array<NT::fr, ARGS_LENGTH> args{};
    for (size_t i = 0; i < args_vec.size(); ++i) {
        args[i] = args_vec[i];
    }
    const NT::fr args_hash = compute_var_args_hash<NT>(args_vec);

    //***************************************************************************
    // Initialize contract related information like private call VK (and its hash),
    // function tree, contract tree, contract address for newly deployed contract,
    // etc...
    //***************************************************************************

    // generate private circuit VK and its hash using circuit with dummy inputs
    // it is needed below:
    //     for constructors - to generate the contract address, function leaf, etc
    //     for private calls - to generate the function leaf, etc
    auto const private_circuit_vk = is_circuit ? utils::get_verification_key_from_file() : utils::fake_vk();

    const NT::fr private_circuit_vk_hash =
        stdlib::recursion::verification_key<CT::bn254>::compress_native(private_circuit_vk, GeneratorIndex::VK);

    ContractDeploymentData<NT> contract_deployment_data{};
    NT::fr contract_tree_root = 0;  // TODO(david) set properly for constructor?
    if (is_constructor) {
        // TODO(david) compute function tree root from leaves
        // create leaf preimage for each function and hash all into tree
        // push to array/vector
        // use variation of `compute_root_partial_left_tree` to compute the root from leaves
        // const auto& function_leaf_preimage = FunctionLeafPreimage<NT>{
        //    .function_selector = function_data.function_selector,
        //    .is_private = function_data.is_private,
        //    .vk_hash = private_circuit_vk_hash,
        //    .acir_hash = acir_hash,
        //};
        std::vector<NT::fr> const function_leaves(MAX_FUNCTION_LEAVES, EMPTY_FUNCTION_LEAF());
        // const NT::fr& function_tree_root = plonk::stdlib::merkle_tree::compute_tree_root_native(function_leaves);

        // TODO(david) use actual function tree root computed from leaves
        // update cdd with actual info
        contract_deployment_data = {
            .deployer_public_key = msg_sender_pub_key,
            .constructor_vk_hash = private_circuit_vk_hash,
            .function_tree_root = plonk::stdlib::merkle_tree::compute_tree_root_native(function_leaves),
            .contract_address_salt = contract_address_salt,
            .portal_contract_address = portal_contract_address,
        };

        // Get constructor hash for use when deriving contract address
        auto constructor_hash = compute_constructor_hash<NT>(function_data, args_hash, private_circuit_vk_hash);

        // Derive contract address so that it can be used inside the constructor itself
        contract_address = compute_contract_address<NT>(
            msg_sender_pub_key, contract_address_salt, contract_deployment_data.function_tree_root, constructor_hash);
        // update the contract address in the call context now that it is known
        call_context.storage_contract_address = contract_address;
    } else {
        const NT::fr& function_tree_root = function_tree_root_from_siblings<NT>(function_data.function_selector,
                                                                                function_data.is_internal,
                                                                                function_data.is_private,
                                                                                private_circuit_vk_hash,
                                                                                acir_hash,
                                                                                function_leaf_index,
                                                                                get_empty_function_siblings());

        // update contract_tree_root with real value
        contract_tree_root = contract_tree_root_from_siblings<NT>(function_tree_root,
                                                                  contract_address,
                                                                  portal_contract_address,
                                                                  contract_leaf_index,
                                                                  get_empty_contract_siblings());
    }

    /**
     * If `is_circuit` is true, we are running a real circuit test and therefore we need to generate a real proof using
     * a private function builder. For the native tests, we are using a random data as public inputs of the private
     * function. As the native private kernel circuit doesn't validate any proofs and we don't currently test
     * multi-iterative kernel circuit, this should be fine.
     */
    PrivateCircuitPublicInputs<NT> private_circuit_public_inputs;
    const NT::Proof private_circuit_proof = utils::get_proof_from_file();
    if (is_circuit) {
        //***************************************************************************
        // Create a private circuit/call using builder, oracles, execution context
        // Generate its proof and public inputs for submission with a TX request
        //***************************************************************************
        Builder private_circuit_builder = Builder();

        DB dummy_db;
        NativeOracle oracle =
            is_constructor
                ? NativeOracle(dummy_db,
                               contract_address,
                               function_data,
                               call_context,
                               contract_deployment_data,
                               msg_sender_private_key)
                : NativeOracle(dummy_db, contract_address, function_data, call_context, msg_sender_private_key);

        OracleWrapper oracle_wrapper = OracleWrapper(private_circuit_builder, oracle);

        FunctionExecutionContext ctx(private_circuit_builder, oracle_wrapper);

        OptionalPrivateCircuitPublicInputs<NT> const opt_private_circuit_public_inputs = func(ctx, args_vec);
        private_circuit_public_inputs = opt_private_circuit_public_inputs.remove_optionality();
        // TODO(suyash): this should likely be handled as part of the DB/Oracle/Context infrastructure
        private_circuit_public_inputs.historic_contract_tree_root = contract_tree_root;

        private_circuit_public_inputs.encrypted_logs_hash = encrypted_logs_hash;
        private_circuit_public_inputs.unencrypted_logs_hash = unencrypted_logs_hash;
        private_circuit_public_inputs.encrypted_log_preimages_length = encrypted_log_preimages_length;
        private_circuit_public_inputs.unencrypted_log_preimages_length = unencrypted_log_preimages_length;
    } else {
        private_circuit_public_inputs = PrivateCircuitPublicInputs<NT>{
            .call_context = call_context,
            .args_hash = args_hash,
            .return_values = {},
            .new_commitments = { NT::fr::random_element() },  // One random commitment
            .new_nullifiers = { NT::fr::random_element() },   // One random nullifier
            .nullified_commitments = {},
            .private_call_stack = {},
            .new_l2_to_l1_msgs = {},
            .encrypted_logs_hash = encrypted_logs_hash,
            .unencrypted_logs_hash = unencrypted_logs_hash,
            .encrypted_log_preimages_length = encrypted_log_preimages_length,
            .unencrypted_log_preimages_length = unencrypted_log_preimages_length,
            .historic_private_data_tree_root = 0,
            .historic_nullifier_tree_root = 0,
            .historic_contract_tree_root = contract_tree_root,
            .historic_l1_to_l2_messages_tree_root = 0,
            .contract_deployment_data = contract_deployment_data,
        };
    }

    const CallStackItem<NT, PrivateTypes> call_stack_item{
        .contract_address = contract_address,
        .function_data = function_data,
        .public_inputs = private_circuit_public_inputs,
    };

    //***************************************************************************
    // Now we can construct the full private inputs to the kernel circuit
    //***************************************************************************

    return std::pair<PrivateCallData<NT>, ContractDeploymentData<NT>>(
    PrivateCallData<NT>{
        .call_stack_item = call_stack_item,
        // TODO(dbanks12): these tests do not test multiple kernel iterations
        // and do not test non-empty callstacks. They should! To have such tests
        // we will need to populate these callstackitem preimages
        // and ensure they match the hashed callstackitems themselves.
        //.private_call_stack_preimages = ,

        .proof = private_circuit_proof,
        .vk = private_circuit_vk,

        .function_leaf_membership_witness = {
            .leaf_index = function_leaf_index,
            .sibling_path = get_empty_function_siblings(),
        },

        .contract_leaf_membership_witness = {
            .leaf_index = contract_leaf_index,
            .sibling_path = get_empty_contract_siblings(),
        },

        .portal_contract_address = portal_contract_address,

        .acir_hash = acir_hash
    },
    contract_deployment_data);
}

/**
 * @brief Perform an initial private circuit call and generate the inputs to private kernel
 *
 * @param is_constructor whether this private circuit call is a constructor
 * @param func the private circuit call being validated by this kernel iteration
 * @param args_vec the private call's args
 * @param encrypted_logs_hash The encrypted logs hash emitted from app circuit.
 * @param unencrypted_logs_hash The unencrypted logs hash emitted from app circuit.
 * @param encrypted_log_preimages_length The length of encrypted logs emitted from app circuit.
 * @param unencrypted_log_preimages_length The length of unencrypted logs emitted from app circuit.
 * @param is_circuit boolean to switch to circuit or native (fake vk and no proof)
 * @return PrivateInputsInit<NT> - the inputs to the private call circuit of an init iteration
 */
PrivateKernelInputsInit<NT> do_private_call_get_kernel_inputs_init(
    bool const is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash,
    NT::fr const& encrypted_log_preimages_length,
    NT::fr const& unencrypted_log_preimages_length,
    bool is_circuit)
{
    //***************************************************************************
    // Initialize some inputs to private call and kernel circuits
    //***************************************************************************
    // TODO(suyash) randomize inputs

    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));

    auto const& [private_call_data, contract_deployment_data] =
        create_private_call_deploy_data(is_constructor,
                                        func,
                                        args_vec,
                                        msg_sender,
                                        encrypted_logs_hash,
                                        unencrypted_logs_hash,
                                        encrypted_log_preimages_length,
                                        unencrypted_log_preimages_length,
                                        is_circuit);

    //***************************************************************************
    // We can create a TxRequest from some of the above data.
    //***************************************************************************
    auto const tx_request = TxRequest<NT>{ .origin = private_call_data.call_stack_item.contract_address,
                                           .function_data = private_call_data.call_stack_item.function_data,
                                           .args_hash = compute_var_args_hash<NT>(args_vec),
                                           .tx_context = TxContext<NT>{
                                               .is_fee_payment_tx = false,
                                               .is_rebate_payment_tx = false,
                                               .is_contract_deployment_tx = is_constructor,
                                               .contract_deployment_data = contract_deployment_data,
                                               .chain_id = 1,
                                           } };

    //***************************************************************************
    // Now we can construct the full private inputs to the kernel circuit
    //***************************************************************************
    PrivateKernelInputsInit<NT> kernel_private_inputs = PrivateKernelInputsInit<NT>{
        .tx_request = tx_request,
        .private_call = private_call_data,
    };

    return kernel_private_inputs;
}


/**
 * @brief Perform an inner private circuit call and generate the inputs to private kernel
 *
 * @param is_constructor whether this private circuit call is a constructor
 * @param func the private circuit call being validated by this kernel iteration
 * @param args_vec the private call's args
 * @param encrypted_logs_hash The encrypted logs hash emitted from app circuit.
 * @param unencrypted_logs_hash The unencrypted logs hash emitted from app circuit.
 * @param encrypted_log_preimages_length The length of encrypted logs emitted from app circuit.
 * @param unencrypted_log_preimages_length The length of unencrypted logs emitted from app circuit.
 * @param public_inputs_encrypted_logs_hash The encrypted logs hash on the output of the previous kernel.
 * @param public_inputs_unencrypted_logs_hash The unencrypted logs hash on the output of the previous kernel.
 * @param public_inputs_encrypted_log_preimages_length The length of encrypted logs on the output of the previous
 * kernel.
 * @param public_inputs_unencrypted_log_preimages_length The length of unencrypted logs on the output of the previous
 * kernel.
 * @param is_circuit boolean to switch to circuit or native (fake vk and no proof)
 * @return PrivateInputsInner<NT> - the inputs to the private call circuit of an inner iteration
 */
PrivateKernelInputsInner<NT> do_private_call_get_kernel_inputs_inner(
    bool const is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash,
    NT::fr const& encrypted_log_preimages_length,
    NT::fr const& unencrypted_log_preimages_length,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_encrypted_logs_hash,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_unencrypted_logs_hash,
    NT::fr const& public_inputs_encrypted_log_preimages_length,
    NT::fr const& public_inputs_unencrypted_log_preimages_length,
    bool is_circuit)
{
    //***************************************************************************
    // Initialize some inputs to private call and kernel circuits
    //***************************************************************************
    // TODO(suyash) randomize inputs

    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));

    auto const& [private_call_data, contract_deployment_data] =
        create_private_call_deploy_data(is_constructor,
                                        func,
                                        args_vec,
                                        msg_sender,
                                        encrypted_logs_hash,
                                        unencrypted_logs_hash,
                                        encrypted_log_preimages_length,
                                        unencrypted_log_preimages_length,
                                        is_circuit);

    const TxContext<NT> tx_context = TxContext<NT>{
        .is_fee_payment_tx = false,
        .is_rebate_payment_tx = false,
        .is_contract_deployment_tx = is_constructor,
        .contract_deployment_data = contract_deployment_data,
    };

    //***************************************************************************
    // We mock a kernel circuit proof to initialize ipnut required by an inner call
    //***************************************************************************

    std::array<NT::fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = private_call_data.call_stack_item.hash();

    auto const& private_circuit_public_inputs = private_call_data.call_stack_item.public_inputs;
    // Get dummy previous kernel
    auto mock_previous_kernel = utils::dummy_previous_kernel(is_circuit);
    // Fill in some important fields in public inputs
    mock_previous_kernel.public_inputs.end.private_call_stack = initial_kernel_private_call_stack;
    mock_previous_kernel.public_inputs.constants = CombinedConstantData<NT>{
        .block_data =
            ConstantHistoricBlockData<NT>{
                .private_data_tree_root = private_circuit_public_inputs.historic_private_data_tree_root,
                .contract_tree_root = private_circuit_public_inputs.historic_contract_tree_root,
            },
        .tx_context = tx_context,
    };
    mock_previous_kernel.public_inputs.is_private = true;
    mock_previous_kernel.public_inputs.end.encrypted_logs_hash = public_inputs_encrypted_logs_hash;
    mock_previous_kernel.public_inputs.end.unencrypted_logs_hash = public_inputs_unencrypted_logs_hash;
    mock_previous_kernel.public_inputs.end.encrypted_log_preimages_length =
        public_inputs_encrypted_log_preimages_length;
    mock_previous_kernel.public_inputs.end.unencrypted_log_preimages_length =
        public_inputs_unencrypted_log_preimages_length;

    //***************************************************************************
    // Now we can construct the full private inputs to the kernel circuit
    //***************************************************************************
    PrivateKernelInputsInner<NT> kernel_private_inputs = PrivateKernelInputsInner<NT>{
        .previous_kernel = mock_previous_kernel,
        .private_call = private_call_data,
    };

    return kernel_private_inputs;
}

/**
 * @brief Validate that the deployed contract address is correct.
 *
 * @details Compare the public inputs new contract address
 * with one manually computed from private inputs.
 * @param private_inputs to be used in manual computation
 * @param public_inputs that contain the expected new contract address
 */
bool validate_deployed_contract_address(PrivateKernelInputsInit<NT> const& private_inputs,
                                        KernelCircuitPublicInputs<NT> const& public_inputs)
{
    auto tx_request = private_inputs.tx_request;
    auto cdd = private_inputs.tx_request.tx_context.contract_deployment_data;

    auto private_circuit_vk_hash = stdlib::recursion::verification_key<CT::bn254>::compress_native(
        private_inputs.private_call.vk, GeneratorIndex::VK);

    auto expected_constructor_hash = compute_constructor_hash(
        private_inputs.private_call.call_stack_item.function_data, tx_request.args_hash, private_circuit_vk_hash);

    NT::fr const expected_contract_address = compute_contract_address(
        cdd.deployer_public_key, cdd.contract_address_salt, cdd.function_tree_root, expected_constructor_hash);

    return (public_inputs.end.new_contracts[0].contract_address.to_field() == expected_contract_address);
}

bool validate_no_new_deployed_contract(KernelCircuitPublicInputs<NT> const& public_inputs)
{
    return (public_inputs.end.new_contracts == CombinedAccumulatedData<NT>{}.new_contracts);
}

}  // namespace aztec3::circuits::kernel::private_kernel::testing_harness
