#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/common.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {

using aztec3::circuits::compute_empty_sibling_path;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionLeafPreimage;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;


using DummyComposer = aztec3::utils::DummyComposer;

using aztec3::utils::zero_array;

// A type representing any private circuit function
// (for now it works for deposit and constructor)
using private_function = std::function<OptionalPrivateCircuitPublicInputs<NT>(
    FunctionExecutionContext<aztec3::circuits::kernel::private_kernel::Composer>&, std::vector<NT::fr> const&)>;

}  // namespace

namespace aztec3::circuits::kernel::private_kernel::testing_harness {

using aztec3::circuits::compute_empty_sibling_path;

// Some helper constants for trees
constexpr size_t MAX_FUNCTION_LEAVES = 1 << aztec3::FUNCTION_TREE_HEIGHT;  // 2^(height-1)
// NOTE: *DO NOT* call hashes in static initializers and assign them to constants. This will fail. Instead, use
// lazy initialization or functions. Lambdas were introduced here.
const auto EMPTY_FUNCTION_LEAF = [] { return FunctionLeafPreimage<NT>{}.hash(); };      // hash of empty/0 preimage
const auto EMPTY_CONTRACT_LEAF = [] { return NewContractData<NT>{}.hash(); };           // hash of empty/0 preimage
constexpr size_t PRIVATE_DATA_TREE_NUM_LEAVES = 1 << aztec3::PRIVATE_DATA_TREE_HEIGHT;  // 2^(height-1)

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
 * @details read requests are siloed by contract address before being
 * inserted into mock private data tree
 *
 * @param contract_address address to use when siloing read requests
 * @param num_read_requests if negative, use random num
 * @return std::tuple<read_requests, read_request_memberships_witnesses, historic_private_data_tree_root>
 */
std::tuple<std::array<NT::fr, READ_REQUESTS_LENGTH>,
           std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH>,
           NT::fr>
get_random_reads(NT::fr const& contract_address, int num_read_requests);

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
    std::array<NT::fr, 2> const& encrypted_logs_hash,
    NT::fr const& encrypted_log_preimages_length,
    bool is_circuit = false);

/**
 * @brief Perform a private circuit call and generate the inputs to private kernel inner circuit
 *
 * @param is_constructor whether this private circuit call is a constructor
 * @param func the private circuit call being validated by this kernel iteration
 * @param args_vec the private call's args
 * @param real_kernel_circuit indicates whether the vk and proof included should be real and usable by real circuits
 * @param encrypted_logs_hash the hash of the encrypted logs to be set in private circuit public inputs
 * @param encrypted_log_preimages_length the length of the encrypted log preimages to be set in private circuit public
 * @return PrivateKernelInputsInner<NT> - the inputs to the private kernel inner circuit
 */
PrivateKernelInputsInner<NT> do_private_call_get_kernel_inputs_inner(
    bool is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, 2> const& encrypted_logs_hash = zero_array<NT::fr, 2>(),
    NT::fr const& encrypted_log_preimages_length = NT::fr(0),
    bool is_circuit = false);

/**
 * @brief Perform a private circuit call and generate the inputs to private kernel init circuit
 *
 * @param is_constructor whether this private circuit call is a constructor
 * @param func the private circuit call being validated by this kernel iteration
 * @param args_vec the private call's args
 * inputs
 * @return PrivateKernelInputsInit<NT> - the inputs to the private kernel init circuit
 */
PrivateKernelInputsInit<NT> do_private_call_get_kernel_inputs_init(
    bool is_constructor,
    private_function const& func,
    std::vector<NT::fr> const& args_vec,
    std::array<NT::fr, 2> const& encrypted_logs_hash = zero_array<NT::fr, 2>(),
    NT::fr const& encrypted_log_preimages_length = NT::fr(0),
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
