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

using DummyBuilder = aztec3::utils::DummyCircuitBuilder;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;


// TODO(jeanmon): the following code will be optimized based on hints regarding matching
// a read request and commitment, i.e., we get pairs i,j such that read_requests[i] == new_commitments[j]
// Relevant task: https://github.com/AztecProtocol/aztec-packages/issues/892
void chop_pending_commitments(DummyBuilder& builder,
                              std::array<NT::fr, MAX_READ_REQUESTS_PER_TX> const& read_requests,
                              std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>,
                                         MAX_READ_REQUESTS_PER_TX> const& read_request_membership_witnesses,
                              std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>& new_commitments)
{
    // chop commitments from the previous call(s)
    for (size_t i = 0; i < MAX_READ_REQUESTS_PER_TX; i++) {
        const auto& read_request = read_requests[i];
        const auto is_transient_read = read_request_membership_witnesses[i].is_transient;

        if (is_transient_read) {
            size_t match_pos = MAX_NEW_COMMITMENTS_PER_TX;
            for (size_t j = 0; j < MAX_NEW_COMMITMENTS_PER_TX; j++) {
                match_pos = (read_request == new_commitments[j]) ? j : match_pos;
            }

            // chop the pending commitment, i.e., replacing with 0.
            // TODO(jeanmon): In addition, we will NOT chop if this is only a read request action without nullifiers.
            if (match_pos != MAX_NEW_COMMITMENTS_PER_TX) {
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
    utils::array_rearrange<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>(new_commitments);
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_ordering(DummyBuilder& builder,
                                                                     PreviousKernelData<NT> const& previous_kernel)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    common_initialise_end_values(previous_kernel, public_inputs);

    // Removing of nullified pending commitments have to happen on the list of commitments which have been accumulated
    // over all iterations of the private kernel. Therefore, we have to target commitments in public_inputs.end
    // Remark: The commitments in public_inputs.end have already been SILOED!
    chop_pending_commitments(builder,
                             public_inputs.end.read_requests,
                             public_inputs.end.read_request_membership_witnesses,
                             public_inputs.end.new_commitments);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel