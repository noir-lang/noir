#include "init.hpp"

#include "aztec3/circuits/abis/function_leaf_preimage.hpp"
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/new_contract_data.hpp>

#include <aztec3/utils/array.hpp>
#include <aztec3/utils/dummy_composer.hpp>
#include <aztec3/circuits/hash.hpp>
#include "aztec3/constants.hpp"

#include <barretenberg/stdlib/merkle_tree/membership.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::ContractLeafPreimage;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;

using aztec3::utils::array_length;
using aztec3::utils::array_pop;
using aztec3::utils::array_push;
using aztec3::utils::is_array_empty;
using aztec3::utils::push_array_to_array;
using DummyComposer = aztec3::utils::DummyComposer;

using aztec3::circuits::compute_constructor_hash;
using aztec3::circuits::compute_contract_address;
using aztec3::circuits::contract_tree_root_from_siblings;
using aztec3::circuits::function_tree_root_from_siblings;

// using plonk::stdlib::merkle_tree::

// // TODO: NEED TO RECONCILE THE `proof`'s public inputs (which are uint8's) with the
// // private_call.call_stack_item.public_inputs!
// CT::AggregationObject verify_proofs(Composer& composer,
//                                     PrivateInputs<CT> const& private_inputs,
//                                     size_t const& num_private_call_public_inputs,
//                                     size_t const& num_private_kernel_public_inputs)
// {
//     CT::AggregationObject aggregation_object = Aggregator::aggregate(
//         &composer, private_inputs.private_call.vk, private_inputs.private_call.proof,
//         num_private_call_public_inputs);

//     Aggregator::aggregate(&composer,
//                           private_inputs.previous_kernel.vk,
//                           private_inputs.previous_kernel.proof,
//                           num_private_kernel_public_inputs,
//                           aggregation_object);

//     return aggregation_object;
// }

void initialise_end_values(PrivateInputs<NT> const& private_inputs, KernelCircuitPublicInputs<NT>& public_inputs)
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
    end.l1_msg_stack = start.l1_msg_stack;

    end.optionally_revealed_data = start.optionally_revealed_data;
}

void contract_logic(DummyComposer& composer,
                    PrivateInputs<NT> const& private_inputs,
                    KernelCircuitPublicInputs<NT>& public_inputs)
{
    const auto private_call_public_inputs = private_inputs.private_call.call_stack_item.public_inputs;
    const auto& storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
    const auto& portal_contract_address = private_inputs.private_call.portal_contract_address;
    const auto& deployer_address = private_call_public_inputs.call_context.msg_sender;
    const auto& contract_deployment_data =
        private_inputs.signed_tx_request.tx_request.tx_context.contract_deployment_data;

    // contract deployment

    // input storage contract address must be 0 if its a constructor call and non-zero otherwise
    auto is_contract_deployment = public_inputs.constants.tx_context.is_contract_deployment_tx;

    auto private_call_vk_hash = stdlib::recursion::verification_key<CT::bn254>::compress_native(
        private_inputs.private_call.vk, GeneratorIndex::VK);

    auto constructor_hash = compute_constructor_hash(private_inputs.signed_tx_request.tx_request.function_data,
                                                     private_call_public_inputs.args,
                                                     private_call_vk_hash);

    if (is_contract_deployment) {
        composer.do_assert(contract_deployment_data.constructor_vk_hash == private_call_vk_hash,
                           "constructor_vk_hash doesn't match private_call_vk_hash");
    }

    auto const new_contract_address = compute_contract_address<NT>(deployer_address,
                                                                   contract_deployment_data.contract_address_salt,
                                                                   contract_deployment_data.function_tree_root,
                                                                   constructor_hash);

    if (is_contract_deployment) {
        // must imply == derived address
        composer.do_assert(storage_contract_address == new_contract_address,
                           "contract address supplied doesn't match derived address");
    } else {
        // non-contract deployments must specify contract address being interacted with
        composer.do_assert(storage_contract_address != 0,
                           "contract address can't be 0 for non-contract deployment related transactions");
    }

    // compute contract address nullifier
    auto const blake_input = new_contract_address.to_field().to_buffer();
    auto const new_contract_address_nullifier = NT::fr::serialize_from_buffer(NT::blake3s(blake_input).data());

    // push the contract address nullifier to nullifier vector
    if (is_contract_deployment) {
        array_push(public_inputs.end.new_nullifiers, new_contract_address_nullifier);
    }

    // Add new contract data if its a contract deployment function
    NewContractData<NT> native_new_contract_data{ new_contract_address,
                                                  portal_contract_address,
                                                  contract_deployment_data.function_tree_root };

    array_push<NewContractData<NT>, KERNEL_NEW_CONTRACTS_LENGTH>(public_inputs.end.new_contracts,
                                                                 native_new_contract_data);

    /* We need to compute the root of the contract tree, starting from the function's VK:
     * - Compute the vk_hash (done above)
     * - Compute the function_leaf: hash(function_selector, is_private, vk_hash, acir_hash)
     * - Hash the function_leaf with the function_leaf's sibling_path to get the function_tree_root
     * - Compute the contract_leaf: hash(contract_address, portal_contract_address, function_tree_root)
     * - Hash the contract_leaf with the contract_leaf's sibling_path to get the contract_tree_root
     */

    // ensure that historic/purported contract tree root matches the one in previous kernel
    auto const& purported_contract_tree_root =
        private_inputs.private_call.call_stack_item.public_inputs.historic_contract_tree_root;
    auto const& previous_kernel_contract_tree_root =
        private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
            .contract_tree_root;
    composer.do_assert(purported_contract_tree_root == previous_kernel_contract_tree_root,
                       "purported_contract_tree_root doesn't match previous_kernel_contract_tree_root");

    // The logic below ensures that the contract exists in the contracts tree
    if (!is_contract_deployment) {
        auto const& computed_function_tree_root = function_tree_root_from_siblings<NT>(
            private_inputs.private_call.call_stack_item.function_data.function_selector,
            true, // is_private
            private_call_vk_hash,
            private_inputs.private_call.acir_hash,
            private_inputs.private_call.function_leaf_membership_witness.leaf_index,
            private_inputs.private_call.function_leaf_membership_witness.sibling_path);

        auto const& computed_contract_tree_root = contract_tree_root_from_siblings<NT>(
            computed_function_tree_root,
            storage_contract_address,
            portal_contract_address,
            private_inputs.private_call.contract_leaf_membership_witness.leaf_index,
            private_inputs.private_call.contract_leaf_membership_witness.sibling_path);

        composer.do_assert(computed_contract_tree_root == purported_contract_tree_root,
                           "computed_contract_tree_root doesn't match purported_contract_tree_root");
    }
}

void update_end_values(DummyComposer& composer,
                       PrivateInputs<NT> const& private_inputs,
                       KernelCircuitPublicInputs<NT>& public_inputs)
{
    const auto private_call_public_inputs = private_inputs.private_call.call_stack_item.public_inputs;

    const auto& new_commitments = private_call_public_inputs.new_commitments;
    const auto& new_nullifiers = private_call_public_inputs.new_nullifiers;

    const auto& is_static_call = private_call_public_inputs.call_context.is_static_call;

    if (is_static_call) {
        // No state changes are allowed for static calls:
        composer.do_assert(is_array_empty(new_commitments) == true, "new_commitments must be empty for static calls");
        composer.do_assert(is_array_empty(new_nullifiers) == true, "new_nullifiers must be empty for static calls");
    }

    const auto& storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;

    {
        // Nonce nullifier
        // DANGER: This is terrible. This should not be part of the protocol. This is an intentional bodge to reach a
        // milestone. This must not be the way we derive nonce nullifiers in production. It can be front-run by other
        // users. It is not domain separated. Naughty.
        array_push(public_inputs.end.new_nullifiers, private_inputs.signed_tx_request.tx_request.nonce);
    }

    { // commitments & nullifiers
        std::array<NT::fr, NEW_COMMITMENTS_LENGTH> siloed_new_commitments;
        for (size_t i = 0; i < new_commitments.size(); ++i) {
            siloed_new_commitments[i] =
                new_commitments[i] == 0
                    ? 0
                    : add_contract_address_to_commitment<NT>(storage_contract_address, new_commitments[i]);
        }

        std::array<NT::fr, NEW_NULLIFIERS_LENGTH> siloed_new_nullifiers;
        for (size_t i = 0; i < new_nullifiers.size(); ++i) {
            siloed_new_nullifiers[i] =
                new_nullifiers[i] == 0
                    ? 0
                    : add_contract_address_to_nullifier<NT>(storage_contract_address, new_nullifiers[i]);
        }

        push_array_to_array(siloed_new_commitments, public_inputs.end.new_commitments);
        push_array_to_array(siloed_new_nullifiers, public_inputs.end.new_nullifiers);
    }

    { // call stacks
        auto& this_private_call_stack = private_call_public_inputs.private_call_stack;
        push_array_to_array(this_private_call_stack, public_inputs.end.private_call_stack);
    }

    // const auto& portal_contract_address = private_inputs.private_call.portal_contract_address;

    // {
    //     const auto& l1_msg_stack = private_call_public_inputs.l1_msg_stack;
    //     std::array<CT::fr, L1_MSG_STACK_LENGTH> l1_call_stack;

    //     for (size_t i = 0; i < l1_msg_stack.size(); ++i) {
    //         l1_call_stack[i] = CT::fr::conditional_assign(
    //             l1_msg_stack[i] == 0,
    //             0,
    //             CT::compress({ portal_contract_address, l1_msg_stack[i] }, GeneratorIndex::L1_MSG_STACK_ITEM));
    //     }
    // }
}

void validate_this_private_call_hash(DummyComposer& composer, PrivateInputs<NT> const& private_inputs)
{
    const auto& start = private_inputs.previous_kernel.public_inputs.end;
    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    const auto popped_private_call_hash = array_pop(start.private_call_stack);
    const auto calculated_this_private_call_hash = private_inputs.private_call.call_stack_item.hash();

    composer.do_assert(
        popped_private_call_hash == calculated_this_private_call_hash,
        "calculated private_call_hash does not match provided private_call_hash at the top of the call stack");
};

void validate_this_private_call_stack(DummyComposer& composer, PrivateInputs<NT> const& private_inputs)
{
    auto& stack = private_inputs.private_call.call_stack_item.public_inputs.private_call_stack;
    auto& preimages = private_inputs.private_call.private_call_stack_preimages;
    for (size_t i = 0; i < stack.size(); ++i) {
        const auto& hash = stack[i];
        const auto& preimage = preimages[i];

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".
        const auto calculated_hash = hash == 0 ? 0 : preimage.hash();
        composer.do_assert(hash != calculated_hash,
                           format("private_call_stack[", i, "] = ", hash, "; does not reconcile"));
    }
};

void validate_inputs(DummyComposer& composer, PrivateInputs<NT> const& private_inputs)
{
    const auto& this_call_stack_item = private_inputs.private_call.call_stack_item;

    composer.do_assert(this_call_stack_item.function_data.is_private == true,
                       "Cannot execute a non-private function with the private kernel circuit");

    const auto& start = private_inputs.previous_kernel.public_inputs.end;

    const NT::boolean is_base_case = start.private_call_count == 0;

    // TODO: we might want to range-constrain the call_count to prevent some kind of overflow errors. Having said that,
    // iterating 2^254 times isn't feasible.

    NT::fr start_private_call_stack_length = array_length(start.private_call_stack);
    NT::fr start_public_call_stack_length = array_length(start.public_call_stack);
    NT::fr start_l1_msg_stack_length = array_length(start.l1_msg_stack);

    // Base Case
    if (is_base_case) {
        // TODO: change to allow 3 initial calls on the private call stack, so a fee can be paid and a gas
        // rebate can be paid.

        composer.do_assert(start_private_call_stack_length == 1, "Private call stack must be length 1");

        composer.do_assert(start_public_call_stack_length == 0, "Public call stack must be empty");
        composer.do_assert(start_l1_msg_stack_length == 0, "L1 msg stack must be empty");

        composer.do_assert(this_call_stack_item.public_inputs.call_context.is_delegate_call == false,
                           "Users cannot make a delegatecall");
        composer.do_assert(this_call_stack_item.public_inputs.call_context.is_static_call == false,
                           "Users cannot make a static call");

        // The below also prevents delegatecall/staticcall in the base case
        composer.do_assert(this_call_stack_item.public_inputs.call_context.storage_contract_address ==
                               this_call_stack_item.contract_address,
                           "Storage contract address must be that of the called contract");

        composer.do_assert(private_inputs.previous_kernel.vk->contains_recursive_proof == false,
                           "Mock kernel proof must not contain a recursive proof");

        // TODO: Assert that the previous kernel data is empty. (Or rather, the verify_proof() function needs a valid
        // dummy proof and vk to complete execution, so actually what we want is for that mockvk to be
        // hard-coded into the circuit and assert that that is the one which has been used in the base case).
    } else {
        // is_recursive_case

        composer.do_assert(private_inputs.previous_kernel.public_inputs.is_private == true,
                           "Cannot verify a non-private kernel snark in the private kernel circuit");
        composer.do_assert(this_call_stack_item.function_data.is_constructor == false,
                           "A constructor must be executed as the first tx in the recursion");
        composer.do_assert(start_private_call_stack_length != 0,
                           "Cannot execute private kernel circuit with an empty private call stack");
    }
}

// NOTE: THIS IS A VERY UNFINISHED WORK IN PROGRESS.
// TODO: decide what to return.
// TODO: is there a way to identify whether an input has not been used by ths circuit? This would help us more-safely
// ensure we're constraining everything.
KernelCircuitPublicInputs<NT> native_private_kernel_circuit(DummyComposer& composer,
                                                            PrivateInputs<NT> const& private_inputs)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    initialise_end_values(private_inputs, public_inputs);

    validate_inputs(composer, private_inputs);

    validate_this_private_call_hash(composer, private_inputs);

    validate_this_private_call_stack(composer, private_inputs);

    update_end_values(composer, private_inputs, public_inputs);

    contract_logic(composer, private_inputs, public_inputs);

    // We'll skip any verification in this native implementation, because for a Local Developer Testnet, there won't
    // _be_ a valid proof to verify!!! auto aggregation_object = verify_proofs(composer,
    //                                         private_inputs,
    //                                         _private_inputs.private_call.vk->num_public_inputs,
    //                                         _private_inputs.previous_kernel.vk->num_public_inputs);

    // TODO: kernel vk membership check!

    // Note: given that we skipped the verify_proof function, the aggregation object we get at the end will just be the
    // same as we had at the start. public_inputs.end.aggregation_object = aggregation_object;
    public_inputs.end.aggregation_object = private_inputs.previous_kernel.public_inputs.end.aggregation_object;

    return public_inputs;
};

} // namespace aztec3::circuits::kernel::private_kernel