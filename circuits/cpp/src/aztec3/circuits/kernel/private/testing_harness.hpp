#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/circuits/abis/read_request_membership_witness.hpp"
#include "aztec3/circuits/abis/tx_request.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/common.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>
#include <cstdint>

namespace {

using aztec3::circuits::compute_empty_sibling_path;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionLeafPreimage;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::ReadRequestMembershipWitness;
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;


using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

// A type representing any private circuit function
// (for now it works for deposit and constructor)
using private_function = std::function<OptionalPrivateCircuitPublicInputs<NT>(
    FunctionExecutionContext<aztec3::circuits::kernel::private_kernel::Builder>&, std::vector<NT::fr> const&)>;

}  // namespace

namespace aztec3::circuits::kernel::private_kernel::testing_harness {

using aztec3::circuits::compute_empty_sibling_path;

// Some helper constants for trees
constexpr size_t MAX_FUNCTION_LEAVES = 1 << aztec3::FUNCTION_TREE_HEIGHT;  // 2^(height-1)
// NOTE: *DO NOT* call hashes in static initializers and assign them to constants. This will fail. Instead, use
// lazy initialization or functions. Lambdas were introduced here.
const auto EMPTY_FUNCTION_LEAF = [] { return FunctionLeafPreimage<NT>{}.hash(); };           // hash of empty/0 preimage
const auto EMPTY_CONTRACT_LEAF = [] { return NewContractData<NT>{}.hash(); };                // hash of empty/0 preimage
constexpr uint64_t PRIVATE_DATA_TREE_NUM_LEAVES = 1ULL << aztec3::PRIVATE_DATA_TREE_HEIGHT;  // 2^(height-1)

inline const auto& get_empty_function_siblings()
{
    static auto EMPTY_FUNCTION_SIBLINGS = []() {
        const auto result = compute_empty_sibling_path<NT, aztec3::FUNCTION_TREE_HEIGHT>(EMPTY_FUNCTION_LEAF());
        return result;
    }();
    return EMPTY_FUNCTION_SIBLINGS;
}

inline const auto& get_empty_contract_siblings()
{
    static auto EMPTY_CONTRACT_SIBLINGS = []() {
        const auto result = compute_empty_sibling_path<NT, aztec3::CONTRACT_TREE_HEIGHT>(EMPTY_CONTRACT_LEAF());
        return result;
    }();
    return EMPTY_CONTRACT_SIBLINGS;
}

/**
 * @brief Get the random read requests and their membership requests
 *
 * @details read requests are siloed by contract address and nonce before being
 * inserted into mock private data tree
 *
 * @param first_nullifier used when computing nonce for unique_siloed_commitments (private data tree leaves)
 * @param contract_address address to use when siloing read requests
 * @param num_read_requests if negative, use random num
 * @return tuple including read requests, their membership witnesses, their transient versions, and the
 * private data tree root that contains all of these randomly created commitments at random leaf indices
 *     std::tuple<
 *      read_requests,
 *      read_request_memberships_witnesses,
 *      transient_read_requests,
 *      transient_read_request_memberships_witnesses,
 *      historic_private_data_tree_root>
 */
std::tuple<std::array<NT::fr, MAX_READ_REQUESTS_PER_CALL>,
           std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>,
           std::array<NT::fr, MAX_READ_REQUESTS_PER_CALL>,
           std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>,
           NT::fr>
get_random_reads(NT::fr const& first_nullifier, NT::fr const& contract_address, int num_read_requests);

/**
 * @brief Create a private call deploy data object
 *
 * @param is_constructor Whether this private call is a constructor call
 * @param func The private circuit (i.e. constructor in case of contract deployment) call
 * @param args_vec Number of args to that private circuit call
 * @param msg_sender The sender of the transaction request
 * @return std::pair<PrivateCallData<NT>, ContractDeploymentData<NT>> - the generated private call data with the
 * contract deployment data
 */
std::pair<PrivateCallData<NT>, ContractDeploymentData<NT>> create_private_call_deploy_data(
    bool is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    NT::address const& msg_sender,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash,
    NT::fr const& encrypted_log_preimages_length,
    bool is_circuit = false);

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
    bool is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash = std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash =
        std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    NT::fr const& encrypted_log_preimages_length = NT::fr(0),
    NT::fr const& unencrypted_log_preimages_length = NT::fr(0),
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_encrypted_logs_hash =
        std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_unencrypted_logs_hash =
        std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    NT::fr const& public_inputs_encrypted_log_preimages_length = NT::fr(0),
    NT::fr const& public_inputs_unencrypted_log_preimages_length = NT::fr(0),
    bool is_circuit = false);

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
    bool is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash = std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash =
        std::array<NT::fr, NUM_FIELDS_PER_SHA256>{},
    NT::fr const& encrypted_log_preimages_length = NT::fr(0),
    NT::fr const& unencrypted_log_preimages_length = NT::fr(0),
    bool is_circuit = false);

/**
 * @brief Validate that the deployed contract address is correct.
 *
 * @details Compare the public inputs new contract address
 * with one manually computed from private inputs.
 * @param private_inputs to be used in manual computation
 * @param public_inputs that contain the expected new contract address
 * @return true or false
 */
bool validate_deployed_contract_address(PrivateKernelInputsInit<NT> const& private_inputs,
                                        KernelCircuitPublicInputs<NT> const& public_inputs);

/**
 * @brief Checks if there is no newly deployed contract
 *
 * @param public_inputs that contain the expected new contract deployment data
 * @return true or false
 */
bool validate_no_new_deployed_contract(KernelCircuitPublicInputs<NT> const& public_inputs);

}  // namespace aztec3::circuits::kernel::private_kernel::testing_harness
