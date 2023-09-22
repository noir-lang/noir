#include "init.hpp"

#include "aztec3/circuits/abis/complete_address.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;

using plonk::stdlib::array_length;
using plonk::stdlib::array_pop;
using plonk::stdlib::array_push;
using plonk::stdlib::is_array_empty;
using plonk::stdlib::push_array_to_array;

using aztec3::circuits::compute_constructor_hash;
using aztec3::circuits::silo_commitment;
using aztec3::circuits::silo_nullifier;

// TODO: NEED TO RECONCILE THE `proof`'s public inputs (which are uint8's) with the
// private_call.call_stack_item.public_inputs!
CT::AggregationObject verify_proofs(Builder& builder, PrivateKernelInputsInner<CT> const& private_inputs)
{
    // compute P0, P1 for private function proof
    CT::AggregationObject aggregation_object =
        Aggregator::aggregate(&builder, private_inputs.private_call.vk, private_inputs.private_call.proof);

    // computes P0, P1 for previous kernel proof
    // AND accumulates all of it in P0_agg, P1_agg
    Aggregator::aggregate(
        &builder, private_inputs.previous_kernel.vk, private_inputs.previous_kernel.proof, aggregation_object);

    return aggregation_object;
}

/**
 * @brief fill in the initial `end` (AccumulatedData) values by copying
 * contents from the previous iteration of this kernel.
 *
 * @param private_inputs contains the information from the previous kernel iteration
 * as well as signed TX request and the private call information
 * @param public_inputs should be empty here since it is being initialized in this call
 */
void initialise_end_values(PrivateKernelInputsInner<CT> const& private_inputs,
                           KernelCircuitPublicInputs<CT>& public_inputs)
{
    // TODO: Ensure public inputs is empty here
    public_inputs.constants = private_inputs.previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other functions
    // within this circuit:
    auto& end = public_inputs.end;
    const auto& start = private_inputs.previous_kernel.public_inputs.end;

    // TODO
    // end.aggregation_object = start.aggregation_object;

    end.new_commitments = start.new_commitments;
    end.new_nullifiers = start.new_nullifiers;

    end.private_call_stack = start.private_call_stack;
    end.public_call_stack = start.public_call_stack;
    end.new_l2_to_l1_msgs = start.new_l2_to_l1_msgs;

    // TODO
    end.new_contracts = start.new_contracts;

    end.optionally_revealed_data = start.optionally_revealed_data;
}

/**
 * @brief Update the AccumulatedData with new commitments, nullifiers, contracts, etc
 * and update its running callstack with all items in the current private-circuit/function's
 * callstack.
 */
void update_end_values(PrivateKernelInputsInner<CT> const& private_inputs, KernelCircuitPublicInputs<CT>& public_inputs)
{
    const auto private_call_public_inputs = private_inputs.private_call.call_stack_item.public_inputs;

    // TODO: private call count
    const auto& new_commitments = private_call_public_inputs.new_commitments;
    const auto& new_nullifiers = private_call_public_inputs.new_nullifiers;

    const auto& is_static_call = private_call_public_inputs.call_context.is_static_call;

    // No state changes are allowed for static calls:
    is_static_call.must_imply(is_array_empty<Builder>(new_commitments) == true);
    is_static_call.must_imply(is_array_empty<Builder>(new_nullifiers) == true);

    // TODO: name change (just contract_address)
    const auto& storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
    const auto& portal_contract_address = private_inputs.private_call.portal_contract_address;
    const auto& contract_deployment_data = private_call_public_inputs.contract_deployment_data;

    {  // contract deployment
        // input storage contract address must be 0 if its a constructor call and non-zero otherwise
        auto is_contract_deployment = public_inputs.constants.tx_context.is_contract_deployment_tx;

        auto private_call_vk_hash = private_inputs.private_call.vk->compress(GeneratorIndex::VK);
        auto constructor_hash = compute_constructor_hash<CT>(private_inputs.private_call.call_stack_item.function_data,
                                                             private_call_public_inputs.args_hash,
                                                             private_call_vk_hash);

        is_contract_deployment.must_imply(contract_deployment_data.constructor_vk_hash == private_call_vk_hash,
                                          "constructor_vk_hash does not match private call vk hash");

        // compute the contract address (only valid if this is a contract deployment)
        auto const contract_address = abis::CompleteAddress<CT>::compute(contract_deployment_data.deployer_public_key,
                                                                         contract_deployment_data.contract_address_salt,
                                                                         contract_deployment_data.function_tree_root,
                                                                         constructor_hash)
                                          .address;

        // must imply == derived address
        is_contract_deployment.must_imply(
            storage_contract_address == contract_address,
            "storage_contract_address must match derived address for contract deployment");

        // non-contract deployments must specify contract address being interacted with
        (!is_contract_deployment)
            .must_imply(storage_contract_address != CT::fr(0),
                        "storage_contract_address must be nonzero for a private function");

        // compute contract address nullifier
        auto blake_input = CT::byte_array(contract_address.to_field());
        auto contract_address_nullifier = CT::fr(CT::blake3s(blake_input));

        // push the contract address nullifier to nullifier vector
        CT::fr const conditional_contract_address_nullifier =
            CT::fr::conditional_assign(is_contract_deployment, contract_address_nullifier, CT::fr(0));
        array_push<Builder>(public_inputs.end.new_nullifiers, conditional_contract_address_nullifier);

        // Add new contract data if its a contract deployment function
        auto const new_contract_data = NewContractData<CT>{
            .contract_address = contract_address,
            .portal_contract_address = portal_contract_address,
            .function_tree_root = contract_deployment_data.function_tree_root,
        };

        array_push<Builder, NewContractData<CT>, MAX_NEW_CONTRACTS_PER_TX>(public_inputs.end.new_contracts,
                                                                           new_contract_data);
    }

    {  // commitments, nullifiers, and contracts
        std::array<CT::fr, MAX_NEW_COMMITMENTS_PER_CALL> siloed_new_commitments;
        for (size_t i = 0; i < new_commitments.size(); ++i) {
            siloed_new_commitments[i] = CT::fr::conditional_assign(
                new_commitments[i] == 0, 0, silo_commitment<CT>(storage_contract_address, new_commitments[i]));
        }
        std::array<CT::fr, MAX_NEW_NULLIFIERS_PER_CALL> siloed_new_nullifiers;
        for (size_t i = 0; i < new_nullifiers.size(); ++i) {
            siloed_new_nullifiers[i] = CT::fr::conditional_assign(
                new_nullifiers[i] == 0, 0, silo_nullifier<CT>(storage_contract_address, new_nullifiers[i]));
        }

        // Add new commitments/etc to AggregatedData
        push_array_to_array<Builder>(siloed_new_commitments, public_inputs.end.new_commitments);
        push_array_to_array<Builder>(siloed_new_nullifiers, public_inputs.end.new_nullifiers);
    }

    {  // call stacks
        // copy the private function circuit's callstack into the AggregatedData
        const auto& this_private_call_stack = private_call_public_inputs.private_call_stack;
        push_array_to_array<Builder>(this_private_call_stack, public_inputs.end.private_call_stack);
    }

    // {
    //     const auto& new_l2_to_l1_msgs = private_call_public_inputs.new_l2_to_l1_msgs;
    //     std::array<CT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> l1_call_stack;

    //     for (size_t i = 0; i < new_l2_to_l1_msgs.size(); ++i) {
    //         l1_call_stack[i] = CT::fr::conditional_assign(
    //             new_l2_to_l1_msgs[i] == 0,
    //             0,
    //             CT::compress({ portal_contract_address, new_l2_to_l1_msgs[i] }, GeneratorIndex::L2_TO_L1_MSG));
    //     }
    // }
}

/**
 * @brief Ensure that the function/call-stack-item currently being processed by the kernel
 * matches the one that the previous kernel iteration said should come next.
 */
void validate_this_private_call_hash(PrivateKernelInputsInner<CT> const& private_inputs)
{
    const auto& start = private_inputs.previous_kernel.public_inputs.end;
    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    const auto this_private_call_hash = array_pop<Builder>(start.private_call_stack);
    const auto calculated_this_private_call_hash = private_inputs.private_call.call_stack_item.hash();

    this_private_call_hash.assert_equal(calculated_this_private_call_hash, "this private_call_hash does not reconcile");
};

/**
 * @brief Ensure that the callstack inputs are consistent.
 *
 * @details The private function circuit will output a callstack containing just hashes
 * of CallStackItems, but the kernel circuit also needs the actual item preimages.
 * So here we just ensure that the callstack preimages in the kernel's private inputs
 * matches the function's CallStackItem hashes.
 */
void validate_this_private_call_stack(PrivateKernelInputsInner<CT> const& private_inputs)
{
    const auto& stack = private_inputs.private_call.call_stack_item.public_inputs.private_call_stack;
    const auto& preimages = private_inputs.private_call.private_call_stack_preimages;
    for (size_t i = 0; i < stack.size(); ++i) {
        const auto& hash = stack[i];
        const auto& preimage = preimages[i];

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".
        const auto calculated_hash = CT::fr::conditional_assign(hash == 0, 0, preimage.hash());

        hash.assert_equal(calculated_hash, format("private_call_stack[", i, "] = ", hash, "; does not reconcile"));
    }
};

void validate_inputs(PrivateKernelInputsInner<CT> const& private_inputs, bool first_iteration)
{
    // this callstack represents the function currently being processed
    const auto& this_call_stack_item = private_inputs.private_call.call_stack_item;

    this_call_stack_item.function_data.is_private.assert_equal(
        true, "Cannot execute a non-private function with the private kernel circuit");

    const auto& start = private_inputs.previous_kernel.public_inputs.end;

    // base case: have not processed any functions yet
    const CT::boolean is_base_case(first_iteration);

    // TODO: we might want to range-constrain the call_count to prevent some kind of overflow errors
    const CT::boolean is_recursive_case = !is_base_case;

    // Grab stack lengths as output from the previous kernel iteration
    // These lengths are calculated by counting entries until a non-zero one is encountered
    // True array length is constant which is a property we need for circuit inputs,
    // but we want to know "length" in terms of how many nonzero entries have been inserted
    CT::fr const start_private_call_stack_length = array_length<Builder>(start.private_call_stack);
    CT::fr const start_public_call_stack_length = array_length<Builder>(start.public_call_stack);
    CT::fr const start_new_l2_to_l1_msgs_length = array_length<Builder>(start.new_l2_to_l1_msgs);

    // Recall: we can't do traditional `if` statements in a circuit; all code paths are always executed. The below is
    // some syntactic sugar, which seeks readability similar to an `if` statement.

    // Base Case
    std::vector<std::pair<CT::boolean, std::string>> const base_case_conditions{
        // TODO: change to allow 3 initial calls on the private call stack, so a fee can be paid and a gas
        // rebate can be paid.
        { start_private_call_stack_length == 1, "Private call stack must be length 1" },
        { start_public_call_stack_length == 0, "Public call stack must be empty" },
        { start_new_l2_to_l1_msgs_length == 0, "L2 to L1 msgs must be empty" },

        { this_call_stack_item.public_inputs.call_context.is_delegate_call == false,
          "Users cannot make a delegatecall" },
        { this_call_stack_item.public_inputs.call_context.is_static_call == false, "Users cannot make a static call" },

        // The below also prevents delegatecall/staticcall in the base case
        { this_call_stack_item.public_inputs.call_context.storage_contract_address ==
              this_call_stack_item.contract_address,
          "Storage contract address must be that of the called contract" },

        { private_inputs.previous_kernel.vk->contains_recursive_proof == false,
          "Mock kernel proof must not contain a recursive proof" }

        // TODO: Assert that the previous kernel data is empty. (Or rather, the verify_proof() function needs a valid
        // dummy proof and vk to complete execution, so actually what we want is for that mockvk to be
        // hard-coded into the circuit and assert that that is the one which has been used in the base case).
        // kernel VK tree contains 16 VKs that would be used to verify kernel proofs depending on the
        // number of public inputs each of them spits out.
        // TODO (later): merkle membership check that the vk from the previous data is present at leaf 0.

        // TODO: verify signed tx request against current function being called
    };
    is_base_case.must_imply(base_case_conditions);

    // Recursive Case
    std::vector<std::pair<CT::boolean, std::string>> const recursive_case_conditions{
        { private_inputs.previous_kernel.public_inputs.is_private == true,
          "Cannot verify a non-private kernel snark in the private kernel circuit" },
        { this_call_stack_item.function_data.is_constructor == false,
          "A constructor must be executed as the first tx in the recursion" },
        { start_private_call_stack_length != 0,
          "Cannot execute private kernel circuit with an empty private call stack" }
        // TODO (later): assert that previous kernel VK matches VK for this input size
        // TODO (later): membership proof of VK
    };
    is_recursive_case.must_imply(recursive_case_conditions);

    // validate constructor hash
    // generate contract address
    // generate contract address nullifier and add to list
    // create new contract data and add to list
    // MAYBE: check other contract deployment data:
    //        function tree, contracts root
}

// NOTE: THIS IS A VERY UNFINISHED WORK IN PROGRESS.
// TODO: decide what to return.
// TODO: is there a way to identify whether an input has not been used by ths circuit? This would help us more-safely
// ensure we're constraining everything.
KernelCircuitPublicInputs<NT> private_kernel_circuit(Builder& builder,
                                                     PrivateKernelInputsInner<NT> const& _private_inputs,
                                                     bool first_iteration)
{
    const PrivateKernelInputsInner<CT> private_inputs = _private_inputs.to_circuit_type(builder);

    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<CT> public_inputs = KernelCircuitPublicInputs<NT>{}.to_circuit_type(builder);

    // Do this before any functions can modify the inputs.
    initialise_end_values(private_inputs, public_inputs);

    validate_inputs(private_inputs, first_iteration);

    validate_this_private_call_hash(private_inputs);

    validate_this_private_call_stack(private_inputs);

    // TODO (later): do we need to validate this private_call_stack against end.private_call_stack?

    update_end_values(private_inputs, public_inputs);

    auto aggregation_object = verify_proofs(builder, private_inputs);

    // TODO: kernel vk membership check!

    public_inputs.end.aggregation_object = aggregation_object;

    public_inputs.set_public();

    return public_inputs.to_native_type<Builder>();
};

}  // namespace aztec3::circuits::kernel::private_kernel
