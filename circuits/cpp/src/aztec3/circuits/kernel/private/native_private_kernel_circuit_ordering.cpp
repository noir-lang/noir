#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/dummy_composer.hpp"

#include <cstddef>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;

using DummyComposer = aztec3::utils::DummyComposer;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;


// TODO(jeanmon): the following code will be optimized based on hints regarding matching
// a read request and commitment, i.e., we get pairs i,j such that read_requests[i] == new_commitments[j]
// Relevant task: https://github.com/AztecProtocol/aztec-packages/issues/892
void chop_pending_commitments(DummyComposer& composer,
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
                composer.do_assert(
                    false,
                    format("transient read request at position [", i, "] does not match any new commitment"),
                    CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
            }
        }
    }

    // Move all zero entries of this array to the end and preserve ordering of other entries
    utils::array_rearrange<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH>(new_commitments);
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_ordering(DummyComposer& composer,
                                                                     PrivateKernelInputsInner<NT> const& private_inputs)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    common_initialise_end_values(private_inputs, public_inputs);

    // TODO(jeanmon): The passed read requests in chop_pending_commitments() will not be from call_stack_item in
    // the final version. The kernel will have to accumulate all read requests of a given transaction.

    // Removing of nullified pending commitments have to happen on the list of commitments which have been accumulated
    // over all iterations of the private kernel. Therefore, we have to target commitments in public_inputs.end
    // Remark: The commitments in public_inputs.end have already been SILOED!
    chop_pending_commitments(composer,
                             private_inputs.private_call.call_stack_item.public_inputs.read_requests,
                             private_inputs.private_call.read_request_membership_witnesses,
                             public_inputs.end.new_commitments);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel