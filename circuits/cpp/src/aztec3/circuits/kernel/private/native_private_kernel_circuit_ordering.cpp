#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <cstddef>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::utils::CircuitResult;

using DummyBuilder = aztec3::utils::DummyCircuitBuilder;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;


// TODO(jeanmon): the following code will be optimized based on hints regarding matching
// a read request and commitment, i.e., we get pairs i,j such that read_requests[i] == new_commitments[j]
// Relevant task: https://github.com/AztecProtocol/aztec-packages/issues/892
void chop_pending_commitments(DummyBuilder& builder,
                              std::array<NT::fr, READ_REQUESTS_LENGTH> const& read_requests,
                              std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> const&
                                  read_request_membership_witnesses,
                              std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH>& new_commitments)
{
    // chop commitments from the previous call(s)
    for (size_t i = 0; i < READ_REQUESTS_LENGTH; i++) {
        const auto& read_request = read_requests[i];
        const auto is_transient_read = (read_request_membership_witnesses[i].leaf_index == NT::fr(-1));

        if (is_transient_read) {
            size_t match_pos = KERNEL_NEW_COMMITMENTS_LENGTH;
            for (size_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) {
                match_pos = (read_request == new_commitments[j]) ? j : match_pos;
            }

            // chop the pending commitment, i.e., replacing with 0.
            if (match_pos != KERNEL_NEW_COMMITMENTS_LENGTH) {
                new_commitments[match_pos] = fr(0);
            } else {
                builder.do_assert(
                    false,
                    format("transient read request at position [", i, "] does not match any new commitment"),
                    CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
            }
        }
    }

    // Move all zero entries of this array to the end and preserve ordering of other entries
    utils::array_rearrange<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH>(new_commitments);
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_ordering(
    DummyBuilder& builder,
    PreviousKernelData<NT> const& previous_kernel,
    std::array<NT::fr, READ_REQUESTS_LENGTH> const& read_requests,
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> const&
        read_request_membership_witnesses)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    common_initialise_end_values(previous_kernel, public_inputs);

    // TODO(jeanmon): The passed read requests in chop_pending_commitments() will not be from call_stack_item in
    // the final version. The kernel will have to accumulate all read requests of a given transaction.

    // Removing of nullified pending commitments have to happen on the list of commitments which have been accumulated
    // over all iterations of the private kernel. Therefore, we have to target commitments in public_inputs.end
    // Remark: The commitments in public_inputs.end have already been SILOED!
    chop_pending_commitments(
        builder, read_requests, read_request_membership_witnesses, public_inputs.end.new_commitments);

    return public_inputs;
};

CircuitResult<KernelCircuitPublicInputs<NT>> native_private_kernel_circuit_ordering_rr_dummy(
    PreviousKernelData<NT> const& previous_kernel)
{
    DummyBuilder builder = DummyBuilder("private_kernel__sim_ordering");

    // TODO(JEANMON): this is a temporary milestone. At a later stage, we will pass "real" read_requests and
    // membership_witnesses
    std::array<fr, READ_REQUESTS_LENGTH> const read_requests{};
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> const
        read_request_membership_witnesses{};

    auto const& public_inputs = native_private_kernel_circuit_ordering(
        builder, previous_kernel, read_requests, read_request_membership_witnesses);
    return builder.result_or_error(public_inputs);
}

}  // namespace aztec3::circuits::kernel::private_kernel