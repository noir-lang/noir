#include "init.hpp"
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;

/**
 * Checks that an array of CT::fr contains `length` contiguous nonzero values, followed by all zeroes.
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
// TODO: move to own file of helper functions.
template <std::size_t SIZE> CT::boolean array_has_length(std::array<CT::fr, SIZE> const& arr, CT::fr const& length)
{
    CT::fr actual_length = 0;
    CT::boolean hit_zero = false;
    for (const auto& e : arr) {
        hit_zero |= e == 0;
        const CT::fr increment = !hit_zero;
        actual_length += increment;
    }
    return actual_length == length;
};

/**
 * Note: doesn't remove the last element from the array; only returns it!
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
template <std::size_t SIZE> CT::fr array_pop(std::array<CT::fr, SIZE> const& arr)
{
    CT::fr popped_value;
    CT::boolean already_popped = false;
    for (size_t i = arr.size() - 1; i != (size_t)-1; i--) {
        CT::boolean is_non_zero = arr[i] != 0;
        popped_value = CT::fr::conditional_assign(!already_popped && is_non_zero, arr[i], popped_value);

        already_popped |= is_non_zero;
    }
    return popped_value;
};

CT::AggregationObject verify_proofs(Composer& composer,
                                    PrivateInputs<CT> const& private_inputs,
                                    size_t const& num_private_call_public_inputs,
                                    size_t const& num_private_kernel_public_inputs)
{
    CT::AggregationObject aggregation_object = Aggregator::aggregate(
        &composer, private_inputs.private_call.vk, private_inputs.private_call.proof, num_private_call_public_inputs);

    Aggregator::aggregate(&composer,
                          private_inputs.previous_kernel.vk,
                          private_inputs.previous_kernel.proof,
                          num_private_kernel_public_inputs);

    return aggregation_object;
}

void validate_private_call_hash(PrivateInputs<CT> const& private_inputs)
{
    const auto private_call_hash = array_pop(private_inputs.start.private_call_stack);
    const auto calculated_private_call_hash = private_inputs.private_call.call_stack_item.hash();

    private_call_hash.assert_equal(calculated_private_call_hash, "private_call_hash does not reconcile");
};

// using abis::private_kernel::PublicInputs;
CT::boolean base_case(PrivateInputs<CT> const& private_inputs, PublicInputs<CT>& public_inputs)
{
    const auto& start = private_inputs.start;
    const CT::boolean is_base_case = start.private_call_count == 0;

    is_base_case.must_imply(
        array_has_length(start.private_call_stack, 1) // TODO: might change to allow 2, so a fee can be paid.
            && array_has_length(start.public_call_stack, 0) &&
            array_has_length(start.contract_deployment_call_stack, 0) && array_has_length(start.l1_call_stack, 0),
        "Invalid arrays for base case.");

    // If we know the length, we can pick an exact index, rather than `pop`:
    // const auto& private_call_hash = start.private_call_stack[0];
    const auto& private_call = private_inputs.private_call;

    public_inputs.constants.is_constructor_recursion = CT::boolean::conditional_assign(
        is_base_case,
        private_inputs.private_call.call_stack_item.function_signature.is_constructor,
        private_inputs.previous_kernel.public_inputs.constants.is_constructor_recursion);

    public_inputs.constants.is_callback_recursion =
        CT::boolean::conditional_assign(is_base_case,
                                        private_inputs.private_call.call_stack_item.function_signature.is_callback,
                                        private_inputs.previous_kernel.public_inputs.constants.is_callback_recursion);

    // TODO: Verify the ECDSA signature!

    is_base_case.must_imply(private_call.call_stack_item.is_delegate_call == false &&
                                private_call.call_stack_item.is_static_call == false,
                            "A user cannot make a delegatecall or staticcall");

    // The below also prevents delegatecall/staticcall
    is_base_case.must_imply(private_call.call_stack_item.call_context.storage_contract_address ==
                                private_call.call_stack_item.function_signature.contract_address,
                            "Storage contract address must be that of the called contract in the base case");

    // TODO: privatelyExecutedCallback logic and checks

    return is_base_case;
};

// TODO: decide what to return.
void private_kernel_circuit(Composer& composer, OracleWrapper& oracle, PrivateInputs<NT> const& _private_inputs)
{
    const PrivateInputs<CT> private_inputs = _private_inputs.to_circuit_type(composer);
    PublicInputs<CT> public_inputs;

    private_inputs.private_call.call_stack_item.function_signature.is_private.assert_equal(
        true, "Cannot execute a non-private function with the private kernel circuit");

    auto aggregation_object = verify_proofs(composer,
                                            private_inputs,
                                            _private_inputs.private_call.vk->num_public_inputs,
                                            _private_inputs.previous_kernel.vk->num_public_inputs);

    validate_private_call_hash(private_inputs);

    const CT::boolean is_base_case = base_case(private_inputs, public_inputs);

    const CT::boolean is_recursive_case = !is_base_case;

    is_recursive_case.must_imply(private_inputs.previous_kernel.public_inputs.is_private == true,
                                 "Cannot verify a non-private kernel snark in the private kernel circuit");
    is_recursive_case.must_imply(private_inputs.private_call.call_stack_item.function_signature.is_callback == false,
                                 "A callback must be executed as the first tx in the recursion");
    is_recursive_case.must_imply(private_inputs.private_call.call_stack_item.function_signature.is_constructor == false,
                                 "A constructor must be executed as the first tx in the recursion");

    // TODO: kernel vk membership check!

    // This line is just to stop the compiler complaining about `oracle` being unused whilst building.
    oracle.generate_random_element().assert_is_not_zero();

    public_inputs.end.aggregation_object = aggregation_object;

    // public_inputs.set_public();
};

} // namespace aztec3::circuits::kernel::private_kernel