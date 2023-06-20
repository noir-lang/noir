#include "common.hpp"

#include "init.hpp"

#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/dummy_composer.hpp"

using DummyComposer = aztec3::utils::DummyComposer;

using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::ContractLeafPreimage;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NewContractData;

using aztec3::utils::array_push;
using aztec3::utils::is_array_empty;
using aztec3::utils::push_array_to_array;
using DummyComposer = aztec3::utils::DummyComposer;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;
using aztec3::circuits::abis::private_kernel::PrivateCallData;

namespace aztec3::circuits::kernel::private_kernel {

void common_validate_call_stack(DummyComposer& composer, PrivateCallData<NT> const& private_call)
{
    const auto& stack = private_call.call_stack_item.public_inputs.private_call_stack;
    const auto& preimages = private_call.private_call_stack_preimages;
    for (size_t i = 0; i < stack.size(); ++i) {
        const auto& hash = stack[i];
        const auto& preimage = preimages[i];

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".
        const auto calculated_hash = hash == 0 ? 0 : preimage.hash();
        composer.do_assert(hash == calculated_hash,
                           format("private_call_stack[", i, "] = ", hash, "; does not reconcile"),
                           CircuitErrorCode::PRIVATE_KERNEL__PRIVATE_CALL_STACK_ITEM_HASH_MISMATCH);
    }
}

/**
 * @brief Validate all read requests against the historic private data root.
 * Use their membership witnesses to do so. If the historic root is not yet
 * initialized, initialize it using the first read request here (if present).
 *
 * @details More info here:
 * - https://discourse.aztec.network/t/to-read-or-not-to-read/178
 * - https://discourse.aztec.network/t/spending-notes-which-havent-yet-been-inserted/180
 *
 * @param composer
 * @param read_requests the commitments being read by this private call
 * @param read_request_membership_witnesses used to compute the private data root
 * for a given request which is essentially a membership check.
 * @param historic_private_data_tree_root This is a reference to the historic root which all
 * read requests are checked against here.
 */
void common_validate_read_requests(DummyComposer& composer,
                                   NT::fr const& storage_contract_address,
                                   std::array<fr, READ_REQUESTS_LENGTH> const& read_requests,
                                   std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>,
                                              READ_REQUESTS_LENGTH> const& read_request_membership_witnesses,
                                   NT::fr const& historic_private_data_tree_root)
{
    // membership witnesses must resolve to the same private data root
    // for every request in all kernel iterations
    for (size_t rr_idx = 0; rr_idx < aztec3::READ_REQUESTS_LENGTH; rr_idx++) {
        const auto& read_request = read_requests[rr_idx];

        // the read request comes un-siloed from the app circuit so we must silo it here
        // so that it matches the private data tree leaf that we are membership checking
        const auto leaf = silo_commitment<NT>(storage_contract_address, read_request);
        const auto& witness = read_request_membership_witnesses[rr_idx];

        // A pending commitment is the one that is not yet added to private data tree
        // A transient read is when we try to "read" a pending commitment
        // We determine if it is a transient read depending on the leaf index from the membership witness
        // Note that the Merkle membership proof would be null and void in case of an transient read
        // but we use the leaf index as a placeholder to detect a transient read.
        const auto is_transient_read = (witness.leaf_index == NT::fr(-1));

        if (read_request != 0 && !is_transient_read) {
            const auto& root_for_read_request =
                root_from_sibling_path<NT>(leaf, witness.leaf_index, witness.sibling_path);
            composer.do_assert(root_for_read_request == historic_private_data_tree_root,
                               format("private data root mismatch at read_request[",
                                      rr_idx,
                                      "] - ",
                                      "Expected root: ",
                                      historic_private_data_tree_root,
                                      ", Read request gave root: ",
                                      root_for_read_request),
                               CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
        }
    }
}

void common_update_end_values(DummyComposer& composer,
                              PrivateCallData<NT> const& private_call,
                              KernelCircuitPublicInputs<NT>& public_inputs)
{
    const auto private_call_public_inputs = private_call.call_stack_item.public_inputs;

    const auto& new_commitments = private_call_public_inputs.new_commitments;
    const auto& new_nullifiers = private_call_public_inputs.new_nullifiers;

    const auto& is_static_call = private_call_public_inputs.call_context.is_static_call;

    if (is_static_call) {
        // No state changes are allowed for static calls:
        composer.do_assert(is_array_empty(new_commitments) == true,
                           "new_commitments must be empty for static calls",
                           CircuitErrorCode::PRIVATE_KERNEL__NEW_COMMITMENTS_NOT_EMPTY_FOR_STATIC_CALL);
        composer.do_assert(is_array_empty(new_nullifiers) == true,
                           "new_nullifiers must be empty for static calls",
                           CircuitErrorCode::PRIVATE_KERNEL__NEW_NULLIFIERS_NOT_EMPTY_FOR_STATIC_CALL);
    }

    const auto& storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;

    // TODO(dbanks12): (spending pending commitments) don't want to silo and output new commitments
    // that have been matched to a new nullifier

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    {  // commitments & nullifiers
        std::array<NT::fr, NEW_COMMITMENTS_LENGTH> siloed_new_commitments;
        for (size_t i = 0; i < new_commitments.size(); ++i) {
            siloed_new_commitments[i] =
                new_commitments[i] == 0 ? 0 : silo_commitment<NT>(storage_contract_address, new_commitments[i]);
        }

        std::array<NT::fr, NEW_NULLIFIERS_LENGTH> siloed_new_nullifiers;
        for (size_t i = 0; i < new_nullifiers.size(); ++i) {
            siloed_new_nullifiers[i] =
                new_nullifiers[i] == 0 ? 0 : silo_nullifier<NT>(storage_contract_address, new_nullifiers[i]);
        }

        push_array_to_array(composer, siloed_new_commitments, public_inputs.end.new_commitments);
        push_array_to_array(composer, siloed_new_nullifiers, public_inputs.end.new_nullifiers);
    }

    {  // call stacks
        const auto& this_private_call_stack = private_call_public_inputs.private_call_stack;
        push_array_to_array(composer, this_private_call_stack, public_inputs.end.private_call_stack);

        const auto& this_public_call_stack = private_call_public_inputs.public_call_stack;
        push_array_to_array(composer, this_public_call_stack, public_inputs.end.public_call_stack);
    }

    {  // new l2 to l1 messages
        const auto& portal_contract_address = private_call.portal_contract_address;
        const auto& new_l2_to_l1_msgs = private_call_public_inputs.new_l2_to_l1_msgs;
        std::array<NT::fr, NEW_L2_TO_L1_MSGS_LENGTH> new_l2_to_l1_msgs_to_insert;
        for (size_t i = 0; i < new_l2_to_l1_msgs.size(); ++i) {
            if (!new_l2_to_l1_msgs[i].is_zero()) {
                // @todo @LHerskind chain-ids and rollup version id should be added here. Right now, just hard coded.
                // @todo @LHerskind chain-id is hardcoded for foundry
                const auto chain_id = fr(31337);
                new_l2_to_l1_msgs_to_insert[i] = compute_l2_to_l1_hash<NT>(storage_contract_address,
                                                                           fr(1),  // rollup version id
                                                                           portal_contract_address,
                                                                           chain_id,
                                                                           new_l2_to_l1_msgs[i]);
            }
        }
        push_array_to_array(composer, new_l2_to_l1_msgs_to_insert, public_inputs.end.new_l2_to_l1_msgs);
    }

    {  // logs hashes
        // See the following thread if not clear:
        // https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426
        const auto& previous_encrypted_logs_hash = public_inputs.end.encrypted_logs_hash;
        const auto& current_encrypted_logs_hash = private_call_public_inputs.encrypted_logs_hash;
        public_inputs.end.encrypted_logs_hash = accumulate_sha256<NT>({ previous_encrypted_logs_hash[0],
                                                                        previous_encrypted_logs_hash[1],
                                                                        current_encrypted_logs_hash[0],
                                                                        current_encrypted_logs_hash[1] });

        const auto& previous_unencrypted_logs_hash = public_inputs.end.unencrypted_logs_hash;
        const auto& current_unencrypted_logs_hash = private_call_public_inputs.unencrypted_logs_hash;
        public_inputs.end.unencrypted_logs_hash = accumulate_sha256<NT>({ previous_unencrypted_logs_hash[0],
                                                                          previous_unencrypted_logs_hash[1],
                                                                          current_unencrypted_logs_hash[0],
                                                                          current_unencrypted_logs_hash[1] });

        // Add log preimages lengths from current iteration to accumulated lengths
        public_inputs.end.encrypted_log_preimages_length = public_inputs.end.encrypted_log_preimages_length +
                                                           private_call_public_inputs.encrypted_log_preimages_length;
        public_inputs.end.unencrypted_log_preimages_length =
            public_inputs.end.unencrypted_log_preimages_length +
            private_call_public_inputs.unencrypted_log_preimages_length;
    }
}

void common_contract_logic(DummyComposer& composer,
                           PrivateCallData<NT> const& private_call,
                           KernelCircuitPublicInputs<NT>& public_inputs,
                           ContractDeploymentData<NT> const& contract_dep_data,
                           FunctionData<NT> const& function_data)
{
    const auto private_call_public_inputs = private_call.call_stack_item.public_inputs;
    const auto& storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
    const auto& portal_contract_address = private_call.portal_contract_address;

    const auto private_call_vk_hash =
        stdlib::recursion::verification_key<CT::bn254>::compress_native(private_call.vk, GeneratorIndex::VK);

    const auto is_contract_deployment = public_inputs.constants.tx_context.is_contract_deployment_tx;

    // input storage contract address must be 0 if its a constructor call and non-zero otherwise
    if (is_contract_deployment) {
        auto constructor_hash =
            compute_constructor_hash(function_data, private_call_public_inputs.args_hash, private_call_vk_hash);

        auto const new_contract_address = compute_contract_address<NT>(contract_dep_data.deployer_public_key,
                                                                       contract_dep_data.contract_address_salt,
                                                                       contract_dep_data.function_tree_root,
                                                                       constructor_hash);

        // Add new contract data if its a contract deployment function
        NewContractData<NT> const native_new_contract_data{ new_contract_address,
                                                            portal_contract_address,
                                                            contract_dep_data.function_tree_root };

        array_push(composer, public_inputs.end.new_contracts, native_new_contract_data);
        composer.do_assert(contract_dep_data.constructor_vk_hash == private_call_vk_hash,
                           "constructor_vk_hash doesn't match private_call_vk_hash",
                           CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONSTRUCTOR_VK_HASH);

        // must imply == derived address
        composer.do_assert(storage_contract_address == new_contract_address,
                           "contract address supplied doesn't match derived address",
                           CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONTRACT_ADDRESS);

        // compute contract address nullifier
        auto const blake_input = new_contract_address.to_field().to_buffer();
        auto const new_contract_address_nullifier = NT::fr::serialize_from_buffer(NT::blake3s(blake_input).data());

        // push the contract address nullifier to nullifier vector
        array_push(composer, public_inputs.end.new_nullifiers, new_contract_address_nullifier);
    } else {
        // non-contract deployments must specify contract address being interacted with
        composer.do_assert(storage_contract_address != 0,
                           "contract address can't be 0 for non-contract deployment related transactions",
                           CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONTRACT_ADDRESS);

        /* We need to compute the root of the contract tree, starting from the function's VK:
         * - Compute the vk_hash (done above)
         * - Compute the function_leaf: hash(function_selector, is_private, vk_hash, acir_hash)
         * - Hash the function_leaf with the function_leaf's sibling_path to get the function_tree_root
         * - Compute the contract_leaf: hash(contract_address, portal_contract_address, function_tree_root)
         * - Hash the contract_leaf with the contract_leaf's sibling_path to get the contract_tree_root
         */

        // The logic below ensures that the contract exists in the contracts tree

        auto const& computed_function_tree_root =
            function_tree_root_from_siblings<NT>(private_call.call_stack_item.function_data.function_selector,
                                                 true,  // is_private
                                                 private_call_vk_hash,
                                                 private_call.acir_hash,
                                                 private_call.function_leaf_membership_witness.leaf_index,
                                                 private_call.function_leaf_membership_witness.sibling_path);

        auto const& computed_contract_tree_root =
            contract_tree_root_from_siblings<NT>(computed_function_tree_root,
                                                 storage_contract_address,
                                                 portal_contract_address,
                                                 private_call.contract_leaf_membership_witness.leaf_index,
                                                 private_call.contract_leaf_membership_witness.sibling_path);

        auto const& purported_contract_tree_root =
            private_call.call_stack_item.public_inputs.historic_contract_tree_root;

        composer.do_assert(
            computed_contract_tree_root == purported_contract_tree_root,
            "computed_contract_tree_root doesn't match purported_contract_tree_root",
            CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    }
}

void common_initialise_end_values(PrivateKernelInputsInner<NT> const& private_inputs,
                                  KernelCircuitPublicInputs<NT>& public_inputs)
{
    public_inputs.constants = private_inputs.previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other functions
    // within this circuit:
    auto& end = public_inputs.end;
    const auto& start = private_inputs.previous_kernel.public_inputs.end;

    end.new_commitments = start.new_commitments;
    end.new_nullifiers = start.new_nullifiers;

    end.private_call_stack = start.private_call_stack;
    end.public_call_stack = start.public_call_stack;
    end.new_l2_to_l1_msgs = start.new_l2_to_l1_msgs;

    end.encrypted_logs_hash = start.encrypted_logs_hash;
    end.unencrypted_logs_hash = start.unencrypted_logs_hash;

    end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;

    end.optionally_revealed_data = start.optionally_revealed_data;
}

}  // namespace aztec3::circuits::kernel::private_kernel