#include "init.hpp"
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::private_kernel::PrivateInputs;

/**
 * Checks that an array of CT::fr contains `length` contiguous nonzero values, followed by all zeroes.
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` counted, you'll need
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
 */
template <std::size_t SIZE> CT::fr array_pop(std::array<CT::fr, SIZE> const& arr)
{
    CT::fr popped_value;
    CT::boolean already_popped = false;
    for (size_t i = arr.size() - 1; i >= 0; --i) {
        CT::boolean is_non_zero = arr[i] != 0;
        popped_value = CT::fr::conditional_assign(!already_popped && is_non_zero, CT::fr(i), popped_value);
        already_popped |= is_non_zero;
    }
    return popped_value;
};

// using abis::private_kernel::PublicInputs;
void base_case(PrivateInputs<CT> const& private_inputs)
{
    const auto& start = private_inputs.start;
    const CT::boolean is_base_case = start.private_call_count == 0;

    // TODO: Validate empty PreviousKernelData? But then the verify_proof function will error.
    // TODO: Can `verify_proof()` fail gracefully and return `false`?

    is_base_case.must_imply(
        array_has_length(start.private_call_stack, 1) && array_has_length(start.public_call_stack, 0) &&
            array_has_length(start.contract_deployment_call_stack, 0) && array_has_length(start.l1_call_stack, 0),
        "Invalid arrays for base case.");

    // If we know the length, we can pick an exact index, rather than `pop`:
    // const auto& private_call_hash = start.private_call_stack[0];
};

// TODO: decide what to return.
void private_kernel_circuit(Composer& composer, OracleWrapper& oracle, PrivateInputs<NT> const& _private_inputs)
{
    const PrivateInputs<CT> private_inputs = _private_inputs.to_circuit_type(composer);

    base_case(private_inputs);

    // This line is just to stop the compiler complaining about `oracle` being unused whilst building.
    oracle.generate_random_element().assert_is_not_zero();
};

} // namespace aztec3::circuits::kernel::private_kernel